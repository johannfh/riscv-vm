help:
    @echo "Available commands:"
    @echo "  build     - Build the program"
    @echo "  clean     - Clean up output directory"
    @echo "  run       - Build and run the program"
    @echo "  hexyl     - Display the output file in hex format using hexyl"

clean:
    @echo "Cleaning up..."
    rm -rf out

build: clean
    @echo "Creating output directory..."
    mkdir -p out
    @echo "Building program..."
    riscv64-unknown-elf-as program.asm -o out/program.o
    @echo "Linking program..."
    riscv64-unknown-elf-ld -T linker.ld out/program.o -o out/program
    @echo "Build complete. Output is in 'out/program'."

run: build
    @echo "Running program..."
    cargo run -- --program out/program


hexyl:
    @echo "Running hexyl on the output file..."
    hexyl -g 4 --endianness little out/program --no-position --no-characters

