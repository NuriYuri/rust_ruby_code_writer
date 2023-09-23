def test(arg)
  case arg
  when 0
    p 0
  when 1
    p 0
    p 1
  when Array, String, Float
    p 3
  else
    p -1
  end
end

case arg
when 1
  p 0
else
  p 3
end

case arg
when 1
  p 0
else
  p 3
  p 4
end