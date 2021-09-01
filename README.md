# huawei-m3-unlocker

Unlock Huawei mediapad m3 lite via brute force. Might also work for other Android devices. No idea.

This program is a reimplementation of https://github.com/Martazza/Huawei-Bootloader-Unlocker in rust (licensed under MIT).

This tool is supposed to unlock a Huawei MediaPad M3. As stated above, it's brute force. Don't expect it to finish in a day. Or even a week.



## Usage:

Requirement is the `fastboot` (android-tools) command and the device being booted in fastboot mode.
To boot your device in fastboot mode you usually keep both vol-down and power pressed right after turning it on.

Run it with exactly **one** (1) argument to start with an offset other than 1000000000000000:
`huawei-m3-unlocker`
or
`huawei-m3-unlocker 1234567890123456` <- if you provide an argument it requires exactly 16 decimal places.

If no starting code (offset) is provided, it tries to load the previously used offset from a file called 'lastcode' in $PWD.



## Additional comments:

Fuck Huawei for leaving us out in the rain like this.

I have no idea if it actually works. For now it's just chugging along.

It was written in Linux. So to run it as a game (e.g. in windows) you might need to do some modifications. Should work fine in most unixes though.

Do with it (mostly) as you wish.
It's licensed as GPL3.
