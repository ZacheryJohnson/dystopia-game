k8s_flag=''

while getopts 'k' flag; do
    case "${flag}" in
        k) k8s_flag='true' ;;
    esac
done

WORKING_DIR=$(pwd)
PROJECT_DIR=$1
cd $PROJECT_DIR/frontend
if [ "$k8s_flag" = "true" ]; then
    npm run build-k8s
else
    npm run build
fi
cd $WORKING_DIR
