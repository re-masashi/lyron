def count(curr:int, end:int):
	if curr<=end:
		print(curr)
		curr = curr +1
		count(curr, end)

count(0,100)