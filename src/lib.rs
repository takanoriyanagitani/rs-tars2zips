use std::fs::File;
use std::path::Path;

use std::io;

use io::BufRead;
use io::BufReader;
use io::Read;

use io::Seek;
use io::Write;

use rs_tar2zip::tar;
use rs_tar2zip::zip;

use zip::ZipWriter;

pub fn items2zip<'a, I, R, W>(items: I, zwtr: &mut ZipWriter<W>) -> Result<(), io::Error>
where
    R: Read + 'a,
    I: Iterator<Item = Result<tar::Entry<'a, R>, io::Error>>,
    W: Write + Seek,
{
    for ritem in items {
        let mut item: tar::Entry<_> = ritem?;
        rs_tar2zip::entry2zip(&mut item, zwtr)?;
    }
    Ok(())
}

pub fn tar2zip<R, W>(ta: R, zwtr: &mut ZipWriter<W>) -> Result<(), io::Error>
where
    R: BufRead,
    W: Write + Seek,
{
    let mut ta = tar::Archive::new(ta);

    let items = ta.entries()?;
    items2zip(items, zwtr)
}

pub fn targz2zip<R, W>(tgz: R, zwtr: &mut ZipWriter<W>) -> Result<(), io::Error>
where
    R: BufRead,
    W: Write + Seek,
{
    let dec = flate2::bufread::GzDecoder::new(tgz);
    let mut ta = tar::Archive::new(dec);

    let items = ta.entries()?;
    items2zip(items, zwtr)
}

#[derive(Clone, Copy)]
pub enum FileType {
    Tar,
    TarGz,
}

pub fn filename2type_default(filename: &str) -> FileType {
    let p: &Path = filename.as_ref();
    let ext: &str = p.extension().and_then(|o| o.to_str()).unwrap_or_default();
    match ext {
        "tgz" => FileType::TarGz,
        "gz" => FileType::TarGz,
        "tar" => FileType::Tar,
        _ => FileType::Tar,
    }
}

pub fn tarname2zip<T, W>(
    filename: &str,
    name2typ: &T,
    zwtr: &mut ZipWriter<W>,
) -> Result<(), io::Error>
where
    T: Fn(&str) -> FileType,
    W: Write + Seek,
{
    let f: File = File::open(filename)?;
    let typ: FileType = name2typ(filename);
    let br = BufReader::new(f);
    match typ {
        FileType::TarGz => targz2zip(br, zwtr),
        FileType::Tar => tar2zip(br, zwtr),
    }
}

pub fn file2sync_all(f: &mut File) -> Result<(), io::Error> {
    f.sync_all()
}

pub fn file2sync_data(f: &mut File) -> Result<(), io::Error> {
    f.sync_data()
}

pub fn file2sync_none(_f: &mut File) -> Result<(), io::Error> {
    Ok(())
}

pub fn tarname2zname_new_from_outdir_default(outdir: String) -> impl Fn(&str, &mut String) {
    move |tarname: &str, zname: &mut String| {
        let tpath: &Path = tarname.as_ref();
        let noext: &str = tpath
            .file_stem()
            .and_then(|o| o.to_str())
            .unwrap_or_default();
        let noextgzp: &Path = noext.as_ref();
        let noextgz: &str = noextgzp
            .file_stem()
            .and_then(|o| o.to_str())
            .unwrap_or_default();
        let sep: &str = std::path::MAIN_SEPARATOR_STR;

        zname.clear();
        zname.push_str(&outdir);
        zname.push_str(sep);
        zname.push_str(noextgz);
        zname.push_str(".zip");
    }
}

pub fn tarnames2zip<I, T, N, F>(
    names: I,
    name2typ: T,
    tarname2zname: N,
    file2sync: F,
) -> Result<(), io::Error>
where
    I: Iterator<Item = String>,
    T: Fn(&str) -> FileType,
    N: Fn(&str, &mut String),
    F: Fn(&mut File) -> Result<(), io::Error>,
{
    let mut zname: String = String::new();
    for filename in names {
        zname.clear();
        tarname2zname(&filename, &mut zname);
        let zfile: File = File::create(&zname)?;
        let mut zw = ZipWriter::new(zfile);
        tarname2zip(&filename, &name2typ, &mut zw)?;
        let mut w: File = zw.finish()?;
        w.flush()?;
        file2sync(&mut w)?;
    }
    Ok(())
}

pub fn tarnames2zip_default<I>(names: I, outdir: String) -> Result<(), io::Error>
where
    I: Iterator<Item = String>,
{
    tarnames2zip(
        names,
        filename2type_default,
        tarname2zname_new_from_outdir_default(outdir),
        file2sync_none,
    )
}

pub fn stdin2tarnames2zip_default(outdir: String) -> Result<(), io::Error> {
    let i = io::stdin();
    let l = i.lock();
    let lines = l.lines();
    let noerr = lines.map_while(Result::ok);
    tarnames2zip_default(noerr, outdir)
}
