# build py36~39 version
conda activate py36
cargo build --example run_with_plugins --release
cp ../target/release/examples/run_with_plugins ./pyflow/
ldd pyflow/run_with_plugins
rm -rf ./build
python3 whl-py36-setup.py bdist_wheel -p linux-x86_64 -d  py36_dist  --python-tag py36

conda activate py37
cargo build --example run_with_plugins --release
cp ../target/release/examples/run_with_plugins ./pyflow/
ldd pyflow/run_with_plugins
rm -rf ./build
python3 whl-py37-setup.py bdist_wheel -p linux-x86_64 -d  py37_dist  --python-tag py37

conda activate py38
cargo build --example run_with_plugins --release
cp ../target/release/examples/run_with_plugins ./pyflow/
ldd pyflow/run_with_plugins
rm -rf ./build
python3 whl-py38-setup.py bdist_wheel -p linux-x86_64 -d  py38_dist  --python-tag py38


rm -rf dist
mkdir dist
cp py36_dist/pyflow-0.1.0-py36-none-linux_x86_64.whl dist/
cp py37_dist/pyflow-0.1.0-py37-none-linux_x86_64.whl dist/
cp py38_dist/pyflow-0.1.0-py38-none-linux_x86_64.whl dist/
