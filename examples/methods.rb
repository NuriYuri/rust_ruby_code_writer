class A
  def test(arg1, arg2 = arg1, *args)
    arg3 = 0
    arg1 = proc { |arg3, arg4| arg3 + arg4 }.call(arg2, *args)
    arg1 = proc { |arg5, arg6| arg5 * arg6 }.call(arg2, *args)
    def arg1.method(arg22, arg23)
      return arg22 + arg23
    end
    return arg3 + arg1
  end

  def test2(arg88)
    test(arg88, 1, arg88).method(4, 5)
  end
end
def test4(some_arg)
  return some_arg * 4
end