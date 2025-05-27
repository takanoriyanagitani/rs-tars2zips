#!/bin/sh

itg0="./sample.d/t0.tar.gz"
itg1="./sample.d/t1.tar.gz"

geninput(){

	echo creating input tar files...

	mkdir -p sample.d

	echo hw0 > ./sample.d/t0t0.txt
	echo hw1 > ./sample.d/t0t1.txt

	echo hw2 > ./sample.d/t1t2.txt
	echo hw3 > ./sample.d/t1t3.txt

	ls ./sample.d/t0*.txt |
		xargs tar \
			--create \
			--verbose \
			--file /dev/stdout |
		gzip --fast |
		dd \
			if=/dev/stdin \
			of="${itg0}" \
			bs=1048576 \
			status=none

	ls ./sample.d/t1*.txt |
		xargs tar \
			--create \
			--verbose \
			--file /dev/stdout |
		gzip --fast |
		dd \
			if=/dev/stdin \
			of="${itg1}" \
			bs=1048576 \
			status=none

}

test -f "${itg0}" || geninput
test -f "${itg1}" || geninput

mkdir -p ./sample.d/out.d

echo converting tar files to zip files...
ls \
	"${itg0}" \
	"${itg1}" |
	cut -d/ -f3- |
	sed -e 's,^,/guest-i.d/,' |
	wazero \
		run \
		-env ENV_OUTPUT_ZIPS_DIR=/guest-o.d \
		-mount "${PWD}/sample.d/out.d:/guest-o.d" \
		-mount "${PWD}/sample.d:/guest-i.d:ro" \
		./rs-tars2zips.wasm

echo
echo listing creatd zip files...

unzip -lv ./sample.d/out.d/t0.zip
unzip -lv ./sample.d/out.d/t1.zip
