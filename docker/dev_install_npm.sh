echo "$INSTALL_NPM"
if [ -n "$INSTALL_NPM" ]
then
  # If INSTALL_NPM is set, we're running in a dev context
  # Install NPM and our dependencies in preparation to run a dev server
  apt install -y nodejs npm
  npm install
else
  # If INSTALL_NPM is not set, we're running in a production context
  # Remove the dev files we copied
  rm -rf "$DEV_PATH"
fi