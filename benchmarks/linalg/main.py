import numpy as np
import os

# read n from N env
n = int(os.environ.get('N', 10))

# Define Hilbert matrix A
A = np.array([[1.0 / (i + j + 1) for j in range(0, n)] for i in range(0, n)])

# Define vector b
b = np.ones(n)

# Solve Ax = b
x = np.linalg.solve(A, b)

# # Round to nearest integer
# x = np.round(x)

# # Make it a vector of integers
# x = x.astype(int)

# Print vertically for easier reading
print("Solution for x:")
for i in range(0, n):
    print(x[i])

print("Check Ax = b")
print(A @ x)