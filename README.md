# Ďíáǩříťíǩád

>I just thought that it's a shame the word `diakritika`
does not have any diacritics in it.


#### Key points
* `diakritika` is a simple daemon that allows you to type your favourite european language's modifiers (`diacritics`)
in a way that does not twist your arm and does not take away your @s and #s and $s. You can now say goodbye to your horrible Czech keyboard!
* Program is controlled with a simple configuration file with hot-reload*.
* Sample config uses Czech letters for US layout and Ukrainian letters for the russian layout.
* The program automatically creates capitalized version of binding by adding Shift key to them.
* It also allows to bind letters on both of your Alts and use bindings like `LAlt+s` to paste `š`


#### Configuration
is simple:
* start a new line
* put the letter you want to type into an `[]`. For example, [ř]
* On the next line key add key combinations that should trigger this letter, one binding per line (see an example below).
> Program automatically detects characters from config:
> Everything with a length of 1 character is a string symbol
> Everything that starts with 0x will be treated as a scancode
> Otherwise assuming that this is a virtual key name without "VK_" and trying to construct a virtual key (ref: src/win/keyboard_vk.rs:87)

⚠ You are also not restricted to simple bindings like alt+s, you can do something like `ctrl`+`n`+`m`+`F2` to paste a shit emoji and it should be fine.

```ini
[á]
alt+a
[č]
alt+c
[ď]
alt+d
[é]
lalt+e
[ě]
ralt+e
[ň]
win+n
[ó]
alt+o
[ř]
alt+r
[š]
alt+s
[ť]
alt+t
[ú]
lalt+u
[ů]
ralt+u
[ý]
alt+y
[ž]
alt+z


[є]
ALT+э
ALT+'
[і]
ALT+ы
ALT+s
[ї]
win+ы
alt+ъ
alt+]

```

***hot reload is not implemented yet**


#### TODO:
* Add logging with `log` instead of prints
* Clean up mixed Debug and Display traits for structures
* Resolve conflicts and don't add conflicting declarations
* Hot reload of last good configuration. Inotify / etc?
* Auto-add program to system startup with admin rights (without admin access the software can't control administrator's applications which is a shame)
* GitHub CI because manually generation builds is pain
* Add support of different things instead of typing letters? for example, running scripts
* I still don't quite like how program handles repeating characters with alt key pressed. and especially with altGr. It works and works great, but not perfect