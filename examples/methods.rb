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

  def test2(arg88, kwarg:, **kwrest)
    test(arg88, 1, kwarg).method(4, 5)
  end
end
def test4(some_arg, kwarg: 4)
  return some_arg * kwarg
end