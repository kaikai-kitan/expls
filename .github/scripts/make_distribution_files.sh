#! /bin/sh
TAG=$1
PRODUCT_NAME=expls  # ご自身のソフトウェア名に変更
RELEASE=$PRODUCT_NAME-$TAG-arm64-darwin

cargo build --release
mkdir -p dist/$RELEASE
cp LICENSE README.md target/release/expls dist/$RELEASE  # target以下のパスもご自身のソフト名に変更
tar cvfz dist/$RELEASE.tar.gz -C dist $RELEASE
