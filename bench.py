def count(curr:int, end:int):
	if curr<=end:
		print(curr)
		curr = curr +1
		count(curr, end)

count(0,100)

i = 0
while i < 10000:
	i = i + 1
	print(i)