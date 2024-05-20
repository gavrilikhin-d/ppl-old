ns = [i * 5 for i in range(1, 20)]
for n in ns:
    # run `N={n} hyperfine --warmup 3 ./target/linalg.out`
    # run `N={n} hyperfine --warmup 3 "python3 main.py"`
