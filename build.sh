#!/bin/bash

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

build_package() {
    local package_dir=$1
    local build_type=$2
    
    echo -e "${YELLOW}Building $package_dir...${NC}"
    (
        cd "$package_dir"
        if [ "$build_type" == "release" ]; then
            cargo build --release
        else
            cargo build
        fi
        cargo test
    )
    echo -e "${GREEN}Successfully built $package_dir${NC}"
}


generate_docs() {
    local package_dir=$1
    
    echo -e "${YELLOW}Generating documentation for $package_dir...${NC}"
    (
        cd "$package_dir"
        cargo doc --no-deps
    )
    echo -e "${GREEN}Documentation generated for $package_dir${NC}"
}


if [ ! -d "lla_plugin_interface" ] || [ ! -d "lla" ]; then
    echo -e "${RED}Error: Script must be run from the project root containing both 'lla_plugin_interface' and 'lla' directories${NC}"
    exit 1
fi


BUILD_TYPE="debug"
GENERATE_DOCS=false

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --release) BUILD_TYPE="release" ;;
        --docs) GENERATE_DOCS=true ;;
        *) echo "Unknown parameter passed: $1"; exit 1 ;;
    esac
    shift
done


build_package "lla_plugin_interface" "$BUILD_TYPE"


build_package "lla" "$BUILD_TYPE"


if [ "$GENERATE_DOCS" = true ] ; then
    generate_docs "lla_plugin_interface"
    generate_docs "lla"
fi

echo -e "${GREEN}Build process completed successfully${NC}"

if [ "$BUILD_TYPE" == "release" ]; then
    echo -e "${YELLOW}Release binaries can be found in target/release/${NC}"
else
    echo -e "${YELLOW}Debug binaries can be found in target/debug/${NC}"
fi

if [ "$GENERATE_DOCS" = true ] ; then
    echo -e "${YELLOW}Documentation can be found in target/doc/${NC}"
fi