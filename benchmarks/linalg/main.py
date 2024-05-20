import numpy as np
import os

from sympy import harmonic, Float

# read n from N env
n = int(os.environ.get('N', 10))

# Define Hilbert matrix A
A = np.array([[1.0 / (i + j + 1) for j in range(0, n)] for i in range(0, n)])

x = np.ones(n)

# Define vector b through the harmonic numbers (this is better)
b = np.array([Float(harmonic(n + i) - harmonic(i)).evalf() for i in range(0, n)], dtype=np.float64)
# b = A @ x

# Solve Ax = b
x_calc = np.linalg.solve(A, b)

# Print vertically for easier reading
print("Solution for x:")
for i in range(0, n):
    print(x_calc[i])

print("|r| = |Ax_calc - b| = ", end="")
print(np.linalg.norm(A @ x_calc - b))

print("|z| = |x - x_calc| = ", end="")
print(np.linalg.norm(x - x_calc))