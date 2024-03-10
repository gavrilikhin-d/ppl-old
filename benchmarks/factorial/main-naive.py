from math import isqrt

print("Solutions to n! + 1 = x^2")

limit = 10000
n = 1
factorial = n
while n <= limit:
	x = factorial + 1
	root = isqrt(x)
	if root * root == x:
		print(f"n = {n}, x = {root}")
	n += 1
	factorial *= n