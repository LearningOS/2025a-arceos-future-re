#!/bin/bash

tmp_file=hashmap_output.txt
grep_content="Memory tests run OK!"

cd arceos/ || exit


rm pflash.img -f
rm disk.img -f 

make pflash_img
make disk_img


make run A=exercises/support_hashmap/ 2>/dev/null | tee $tmp_file

output=$(grep -a "$grep_content" ./$tmp_file | tail -n1)

rm -rf $tmp_file 

if [[ -z "$output" ]]; then
    echo "support_hashmap default"
    exit 1
else 
    echo "support_hashmap pass"
    exit 0
fi
