p /reg\/exp/,
  %r{/kit/scripts/(.*)},
  %w[.png .PNG .jpg],
  %i[R2 L2],
  %I[R2 L2 #{0}],
  %x{echo 0},
  `echo 0`,
  "test #{0}}",
  %Q{ff #{0}\}},
  :"comeplex sym #{0}",
  :regular_sym,
  { a: 0, "b#@a": 1 }
p <<~EODSP
    The game crashed!#{0}
    The error is stored in Error.log.
  EODSP
p <<-EOSHADER
    uniform vec4 tone;#{0}
  EOSHADER
p <<-`CMD`
    echo 0#{0}
  CMD
p <<~`CMD`
    echo 0#{0}
  CMD
p <<~'EODSP'
    The game crashed!#{0}
    The error is stored in Error.log.
  EODSP
p <<-'EOSHADER'
    uniform vec4 tone;#{0}
  EOSHADER
