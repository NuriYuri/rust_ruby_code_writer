module Test
  A = 0
  A = 1
  B = "C"
  module C
    D = :D
    E = 3.3
  end
end
class A
end
module Test::A
  module ::A
  end
end
PI = 3.14
F = {} # <= Not a simple literal so will not appear in the exploration list