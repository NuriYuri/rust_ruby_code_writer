module Test
  # This class has documentation
  class A
    # Documentation for attr
    # @return [String]
    attr_accessor :wololo
    # Documentation for other attr
    attr_reader :test

    # Documentation for Sub Class
    # With several lines
    class B
      # Single line doc for method
      def c
        return
      end
    end
  end
  # This documentation is lost forever because it's not meant to be documented several times
  class A
    # Super constant
    A = 0

    private
    # Some comments for whatever reason

    # The method from A
    # @return [Integer]
    def a
      return A
    end
  end
end