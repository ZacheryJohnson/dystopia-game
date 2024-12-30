RUST_PROJECT_NAME="dys-matchvisualizer"
OUT_DIR="./.wasm_out"
BIN_NAME="matchvisualizer"

WEBAPP_PUBLIC_DIR_PATH="dys-svc-webapp/frontend/public"
WEBAPP_INTERNAL_DIR_PATH="dys-svc-webapp/frontend/src/assets"

echo "Building project with cargo..."
cargo build -p $RUST_PROJECT_NAME --release --target wasm32-unknown-unknown || exit 1

echo "Binding wasm..."
mkdir $OUT_DIR
wasm-bindgen --target web \
    --out-dir $OUT_DIR \
    --out-name $BIN_NAME \
    ./target/wasm32-unknown-unknown/release/$RUST_PROJECT_NAME.wasm || exit 2

echo "Optimizing wasm output... (this may take many seconds)"
wasm-opt -Oz -o $OUT_DIR/${BIN_NAME}_opt.wasm $OUT_DIR/${BIN_NAME}_bg.wasm || exit 3

echo "Removing unoptimized wasm artifact..."
rm $OUT_DIR/${BIN_NAME}_bg.wasm
rm $OUT_DIR/${BIN_NAME}_bg.wasm.d.ts

cp $OUT_DIR/matchvisualizer_opt.wasm ./$WEBAPP_PUBLIC_DIR_PATH/matchvisualizer_opt.wasm
cp $OUT_DIR/matchvisualizer.d.ts ./$WEBAPP_INTERNAL_DIR_PATH/matchvisualizer.d.ts
cp $OUT_DIR/matchvisualizer.js ./$WEBAPP_INTERNAL_DIR_PATH/matchvisualizer.js

rm -rf $OUT_DIR

echo "Done!"