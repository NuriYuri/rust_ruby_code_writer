module Test
  class A
    def a

    end

    private
    def b

    end
  end
  class B
    def c

    end
  end

  class A
    B = 6
    def c
    end
  end
  class D
    class E
      def o
      end
    end
  end
  class D
    class F
    end
  end
end
module Test
  class D
    class G
    end
  end
end