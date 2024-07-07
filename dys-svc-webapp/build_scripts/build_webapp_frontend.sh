WORKING_DIR=$(pwd)
PROJECT_DIR=$1
cd $PROJECT_DIR/frontend
npm run build
cd $WORKING_DIR
