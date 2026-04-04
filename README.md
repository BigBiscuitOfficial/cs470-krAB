# SETUP INSTRUCTIONS

## First install rustup by running the following command and instructions listed

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Next export llvm path and clang args, either by including this in your .bashrc/.zshrc depending on your shell, or simply running it only for the current session.
```bash
export LIBCLANG_PATH=/scratch/mannlk/llvm/clang+llvm-14.0.0-x86_64-linux-gnu-ubuntu-18.04/lib
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-redhat-linux/8/include"
```

## After that, simply use the helper script to see our scaling results
```bash
./build-and-run.sh
```

## Further helper scripts to be implemented. They will showcase the data calculated by the engine and increased efficiency.
