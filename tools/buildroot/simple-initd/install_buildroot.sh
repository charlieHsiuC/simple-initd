#!/bin/bash

# get buildroot directory path
buildroot_dir=$(realpath $1)

# get simple-initd directory path
config_dir=$(realpath $(dirname $0))
simple_initd_dir=$(realpath $config_dir/../../..)

echo "buildroot_dir: $buildroot_dir"
echo "config_dir: $config_dir"
echo "simple_initd_dir: $simple_initd_dir"

# update SIMPLE_INITD_SITE in tools/buildroot/simple-initd/simple-initd.mk
sed -i "s|SIMPLE_INITD_SITE = .*|SIMPLE_INITD_SITE = $simple_initd_dir|" $config_dir/simple-initd.mk

# copy tools/buildroot/simple-initd to buildroot/package/simple-initd
cp -r $config_dir $buildroot_dir/package

# update package/simple-initd/Config.in to menu "System tools" in buildroot/package/Config.in
# check if "package/simple-initd/Config.in" not exists in buildroot/package/Config.in
if ! grep -q "package/simple-initd/Config.in" $buildroot_dir/package/Config.in; then
    echo "package/simple-initd/Config.in not exists in buildroot/package/Config.in, add it"
    sed -i "/menu \"System tools\"/a \\\tsource \"package/simple-initd/Config.in\"" $buildroot_dir/package/Config.in
fi
