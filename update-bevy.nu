ls examples/**/*.toml | get name | each { |it| open $it | upsert dependencies.bevy.rev {|v| "9f8bdeeeb94d8c3ac2e24e31fda4b32792151287"} | save $it }