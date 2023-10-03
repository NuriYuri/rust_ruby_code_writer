a = begin
  p 0
  5
end
b = if a == 5
  3
elsif a== 2
  2
else
  1
end
c = if a && !b.next
  2
else
  3
end