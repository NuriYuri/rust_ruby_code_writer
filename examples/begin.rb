def test
  begin
    p 0
  end
end

proc do
  begin
    p 0
  rescue Error => e
    p e
  rescue Exception => e
    p e
  else
    p 1
  ensure
    p 2
  end
end

def test
  p 0
rescue
  p 1
end

def test2
  p 0
ensure
  p 1
end


def test3
  p 0
rescue => e
  p e
else
  p 2
ensure
  p 1
end


begin
  p 0
  p 1
rescue Error => e
  p e
  p e
rescue Exception => e
  p e
  p e
else
  p 1
  p 3
ensure
  p 2
  p 4
end