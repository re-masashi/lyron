def count(curr, e)
   if curr<e then
     curr = curr+1
     # puts curr
     count(curr, e)
   end
end

count(1,100000)
