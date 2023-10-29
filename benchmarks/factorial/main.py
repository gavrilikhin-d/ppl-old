import multiprocessing
import sys
from math import isqrt

# Increase the recursion limit
sys.setrecursionlimit(50000)  # Adjust the limit as needed

# Check if a number is a perfect square


def is_perfect_square(x):
    root = isqrt(x)
    return root * root == x

# Find solutions to the equation n! + 1 = x^2 in parallel


def find_solutions(limit):
    pool = multiprocessing.Pool()
    results = pool.map(find_solution, range(1, limit + 1), chunksize=10)
    pool.close()
    pool.join()
    return [(n, x) for n, x in results if x > 1]


# Memoization dictionary for factorial values
fact_cache = {}


def factorial_memo(n):
    if n < 0:
        raise ValueError("Factorial is not defined for negative numbers")
    if n == 0:
        return 1
    if n not in fact_cache:
        fact_cache[n] = n * factorial_memo(n - 1)
    return fact_cache[n]


def find_solution(n):
    fact = factorial_memo(n)
    x_square = fact + 1
    x = isqrt(x_square)
    if is_perfect_square(x_square):
        return (n, x)
    else:
        return (-1, -1)  # Placeholder for no solution


if __name__ == "__main__":
    limit = 10000  # Adjust the limit as needed
    solutions = find_solutions(limit)

    print("Solutions to n! + 1 = x^2:")
    for n, x in solutions:
        print(f"n = {n}, x = {x}")
