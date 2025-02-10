#!/bin/sh
RED='\033[1;31m'
BLUE='\033[1;34m'
MAG='\033[1;35m'
GRAY='\033[0;90m'
RESET='\033[0m'

echo "${GRAY}(c) 2024 Marcio Dantas${RESET}"

set -e

# echo "${BLUE}Downloading Pile${RESET}"

# if [ -d "./pile" ]; then
#     rm -rf pile
# fi

# git clone --depth 1 --branch master https://github.com/igotfr/pile.git

# cd Pile

echo "${BLUE}Building Pile${RESET}"

cargo build --release

if [ $? -eq 0 ]; then
    echo "${BLUE}Installing Pile${RESET}"

    EXEC_NAME=$(basename $(pwd))
    mv target/release/$EXEC_NAME .
    sudo cp -r pile /usr/local/bin/

    echo "${GRAY}Pile Installed Successfully${RESET}"
    echo "${MAG}Thank You!${RESET}"
else
    echo "${RED}Build Failed${RESET}"
fi