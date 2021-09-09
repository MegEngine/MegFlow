# build py36~38 version
CONDA_BASE=$(conda info --base)
source $CONDA_BASE/etc/profile.d/conda.sh
conda activate py36
cargo build --example run_with_plugins --release
cp ../target/release/examples/run_with_plugins ./pyflow/run_with_plugins_inner
rm -rf ./build
py='py36' python3 setup.py bdist_wheel -p linux-x86_64 -d  py36_dist

conda activate py37
cargo build --example run_with_plugins --release
cp ../target/release/examples/run_with_plugins ./pyflow/run_with_plugins_inner
rm -rf ./build
py='py37' python3 setup.py bdist_wheel -p linux-x86_64 -d  py37_dist

conda activate py38
cargo build --example run_with_plugins --release
cp ../target/release/examples/run_with_plugins ./pyflow/run_with_plugins_inner
rm -rf ./build
py='py38' python3 setup.py bdist_wheel -p linux-x86_64 -d  py38_dist


rm -rf dist
mkdir dist
cp py36_dist/pyflow-0.1.0-py36-none-linux_x86_64.whl dist/
cp py37_dist/pyflow-0.1.0-py37-none-linux_x86_64.whl dist/
cp py38_dist/pyflow-0.1.0-py38-none-linux_x86_64.whl dist/
rm -rf py36_dist py37_dist py38_dist
