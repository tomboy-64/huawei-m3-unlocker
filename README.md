# huawei-m3-unlocker
Unlock Huawei mediapad m3 lite via brute force. Might also work for other Android devices. No idea.

This program is a reimplementation of https://github.com/Martazza/Huawei-Bootloader-Unlocker in rust (licensed under MIT).

This tool is supposed to unlock a Huawei MediaPad M3.
Run it with exactly 1 argument to start with an offset other than 1000000000000000.
Else it tries to load the previously used offset from 'lastcode' in $PWD.

Fuck Huawei for leaving us out in the rain like this.



I have no idea if it actually works. For now it's just running. It was written in Linux. So to run it as a game (e.g. in windows) you might need to do some modifications. Should work fine in most unixes though.

Do with it (mostly) as you wish.
It's licensed as GPL3.
