#!/bin/bash    

# the directory of the script
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# the temp directory used, within $DIR
# omit the -p parameter to create a temporal directory in the default location
WORK_DIR=`mktemp -d -p "$DIR"`

# check if tmp dir was created
if [[ ! "$WORK_DIR" || ! -d "$WORK_DIR" ]]; then
  echo "Could not create temp dir"
  exit 1
fi

# deletes the temp directory
function cleanup {      
  rm -rf "$WORK_DIR"
  echo "Deleted temp working directory $WORK_DIR"
}

# register the cleanup function to be called on the EXIT signal
trap cleanup EXIT

REPO=$1
REPO_DIR="$WORK_DIR/$REPO"

set -e
git clone --depth=1 https://github.com/$REPO $REPO_DIR
python3 -m venv $REPO_DIR/venv
source $REPO_DIR/venv/bin/activate

if [ -z "$2" ];
then (cd $REPO_DIR && pip install $REPO_DIR);
else (cd $REPO_DIR && pip install -r $REPO_DIR/$2);
fi

mkdir -p output/$REPO

hyperfine --export-json output/$REPO/result.json --warmup 3 "pytest $REPO_DIR/tests --collect-only" "target/release/rytest $REPO_DIR/tests --collect-only"