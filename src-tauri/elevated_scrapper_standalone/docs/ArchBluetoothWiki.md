=== Dual boot pairing ===

To pair devices on dual boot setups you need to change the pairing keys on your Linux install so that they are consistent with what Windows or macOS is using.

This page only describes the manual method of doing so. To automate the process, see the [https://github.com/x2es/bt-dualboot bt-dualboot project] and the related repositories.  For a semi-automated process, use the [https://github.com/nbanks/bluetooth-dualboot bluetooth-dualboot script] which does not edit any files, but it helps you run the right commands and cut-and-paste the correct values.

==== Setup ====

To do this, first pair your device on your Arch Linux install. Then reboot into the other OS and pair the device. Now you need to extract the pairing keys, but first switch off the Bluetooth devices to prevent any connection attempts.

{{Note|Some Logitech devices, such as the [[Logitech MX Master]] and Logitech 604 Lightspeed, increment the MAC address by one every time that the device is paired with a new system. You should determine whether this is the case, so that it can be accounted for at the end of the process.}}

==== For Windows ====

You can extract your Bluetooth keys on either Linux or Windows:

===== Extracting on Windows =====

First, boot into Windows.

The registry key containing the link keys may only be accessed by the [https://docs.microsoft.com/en-us/windows/security/identity-protection/access-control/local-accounts#system SYSTEM account], which cannot be logged into. Therefore, you will need Microsoft's [https://docs.microsoft.com/en-us/sysinternals/downloads/psexec PsExec] tool from the official Windows Sysinternals site in order to run {{ic|regedit.exe}} as {{ic|SYSTEM}}.

Download [https://download.sysinternals.com/files/PSTools.zip PsTools], and extract {{ic|PsExec64.exe}}.

In an administrator instance of a [https://docs.microsoft.com/en-us/windows-server/administration/windows-commands/windows-commands#command-shell-overview command shell], from the location of the extracted EXE, launch the registry editor:

 .\PsExec64.exe -s -i regedit.exe

In the registry editor, navigate to the following registry key:

 HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Services\BTHPORT\Parameters\Keys

Within this registry key is one subkey per Bluetooth adapter, named by MAC address. If there are multiple subkeys, and you are unsure of which to use, follow [https://answers.microsoft.com/en-us/windows/forum/all/how-to-view-addresses-of-generic-bluetooth-input/36ead4de-e62f-43a0-b76d-b9708460ddfd this guide] to find the MAC address for the desired Bluetooth adapter.

In the desired adapter's registry key, there is a name-value pair for each paired device, with the name being its MAC address. Additionally, you might see some subkeys named by MAC addresses, each containing name-value pairs with names like {{ic|LTK}} or {{ic|IRK}}. These subkeys (if any) are for Bluetooth 5.1 devices. If the device you're trying to share has a subkey, it is a Bluetooth 5.1 device. If it does not have a subkey, only a name-value pair, it is not a Bluetooth 5.1 device.

Right click on the adapter's registry key and export it as a ''.reg'' file. This is a text file that you can copy keys from. As mentioned, it contains pairing keys in name-value pairs for non-Bluetooth 5.1 devices, and pairing keys (and some other information) in per-device subkeys for Bluetooth 5.1 devices. Make this file available to your Linux installation and reboot into it.

If the device you want to share is ''not'' a Bluetooth 5.1 device, jump to [[#Saving the configuration]]. If it is a Bluetooth 5.1 device, you need to make some modifications to the pairing keys and the associated information before finishing up. Refer to [[#Preparing Bluetooth 5.1 Keys]] to see how.

===== Extracting on Linux =====

{{Note|If your Windows partition is encrypted with Bitlocker, you will not be able to access it from Linux using chntpw.}}

Boot into Arch. Install {{Pkg|chntpw}}. Mount your windows system drive. Change to the registry hive directory and start {{ic|chntpw}} on the SYSTEM hive:

{{Tip|If {{Pkg|rlwrap}} is installed, {{ic|chntpw}} can be run as {{ic|rlwrap chntpw}} to provide readline-style command-lne editing and history.}}

 $ cd ''/path/to/windows/system''/Windows/System32/config
 $ chntpw -e SYSTEM

Inside the {{ic|chntpw}} environment, run:

 > cd CurrentControlSet\Services\BTHPORT\Parameters\Keys

Instead of {{ic|CurrentControlSet}}, you may see {{ic|ControlSet00''X''}} (check using {{ic|ls}}); use this instead:

 > cd ControlSet00''X''\Services\BTHPORT\Parameters\Keys

There will probably be just one subkey, whose name is your Bluetooth adapter's MAC address.
Show it with {{ic|ls}} and {{ic|cd}} into it:

 > ls
 > cd ''your-adapter's-mac-address''

The subkey names under that adapter are the MAC addresses of the ''devices'' the adapter is paired to.
Show them with {{ic|ls}}, then {{ic|cd}} into each of those you want to dual-pair:

 > ls
 > cd ''your-device's-mac-address''

If this is ''not'' a Bluetooth 5.1 device, you will only see the pairing key:

{{hc|> ls|
Node has 0 subkeys and 1 values
size  type        value name    [value if type DWORD]
16    REG_BINARY <ab12cd34ef56>
}}

If so, show a hexdump of your device's key using {{ic|hex}}:

{{hc|> hex ab12cd34ef56|
:00000 XX XX XX XX XX XX XX XX XX XX XX XX XX XX XX XX (some other chars)
}}

The "XX"s are the pairing key. Make note of which keys map to which MAC addresses.

If this ''is'' a Bluetooth 5.1 device, then you will see several keys corresponding to the one device:

{{bc|
Node has 0 subkeys and 8 values
  size     type              value name             [value if type DWORD]
    16  3 REG_BINARY         <LTK>
     4  4 REG_DWORD          <KeyLength>               16 [0x10]
     8  b REG_QWORD          <ERand>
     4  4 REG_DWORD          <EDIV>                 37520 [0x9290]
    16  3 REG_BINARY         <IRK>
     8  b REG_QWORD          <Address>
     4  4 REG_DWORD          <AddressType>              1 [0x1]
     4  4 REG_DWORD          <AuthReq>                 45 [0x2d]
}}

Refer to [[#Preparing Bluetooth 5.1 Keys]] to see how to use these, using {{ic|hex ''value_name''}} to obtain the requested values.

Finally, to import the key(s) into your Linux installation, proceed to [[#Saving the configuration]].

==== For macOS ====

Boot into macOS:

* For macOS Monterey or newer:
*# Open Keychain Access and search for Bluetooth.
*# Sort by date.
*# If you've recently removed and reconnected the device then you can simply sort the keys by date modified and pick the latest. It is probably called MobileBluetooth (for older Bluetooth devices) or is just an UUID (for Bluetooth 5.1+).
*# Double click on the entry. Check that the MAC address in the Account field matches the MAC address of your device.
*# Click the "Show password" checkbox. You will now need to enter your password, twice.
*# Copy the text in the password field, it's actually an XML file ({{ic|⌘+a}} {{ic|⌘+c}})
*# Paste the text in {{ic|bt_keys.txt}} in your home directory.
* For High Sierra or newer, run the following in a terminal: {{bc|# defaults read /private/var/root/Library/Preferences/com.apple.bluetoothd.plist LinkKeys > ~/bt_keys.txt}}
* For Sierra or older, run the following in a terminal: {{bc|# defaults read /private/var/root/Library/Preferences/blued.plist LinkKeys > ~/bt_keys.txt}}

The {{ic|~/.bt_keys.txt}} file now contains the established Bluetooth keys. For older versions of macOS (High Sierra and older) you will have to reverse the keys before proceeding. For example, {{ic|98 54 2f aa bb cc dd ee ff gg hh ii jj kk ll mm}} becomes {{ic|MM LL KK JJ GG FF EE DD CC BB AA 2F 54 98}}.

{{Note|The reversal can be done with the following [[Python]] code:
{{bc|1=
>>> key = "98 54 2f aa bb cc dd ee ff gg hh ii jj kk ll mm"
>>> " ".join(reversed(key.split()))
}}
}}

If this is a Bluetooth 5.1 device, then there will be more than one key corresponding to one device. Refer to [[#Preparing Bluetooth 5.1 Keys]] to see how to use these.

Finally, to import the key(s) into your Linux installation, reboot into Linux and proceed to [[#Saving the configuration]].

==== Preparing Bluetooth 5.1 Keys ====

{{Expansion|We are still working on getting a comprehensive idea of how these work across vendors. For now, documenting specific devices' compatibility with these methods is helpful - especially non-Logitech data points.|Talk:Bluetooth#Bluetooth 5.1 Devices}}

If you observed the presence of Bluetooth 5.1 keys while following [[#For Windows]] or [[#For macOS]], you must apply certain transformations to their values before importing them into Linux. Create the requested files with their appropriate contents, for installation in [[#Saving the configuration]]. This process will depend on the device, and some of the values have to be manipulated; code utilities for doing so are provided below.

{| class="wikitable"
! Device !! Source Key and Transformations (Windows) !! Source Key and Transformations (macOS) || Destination Key File
|-
|rowspan="3"|
* Logitech MX Anywhere 3S
* Logitech MX Master 3
* Logitech MX Master 3S
* Logitech MX Keys
* Logitech MX Keys Mini
* Logitech MX Mechanical
* Xbox One S Wireless Controller
|
* Copy {{ic|IRK}}.
* Remove the spaces between the hex octets.
| {{Grey|?}}
| {{ic|IdentityResolvingKey.Key}}
|-
|
* Copy {{ic|LTK}}.
* Remove the spaces between the hex octets.
| {{Grey|?}}
| {{ic|SlaveLongTermKey.Key}} and {{ic|PeripheralLongTermKey.Key}}
|-
|{{ic|ERand}} and {{ic|EDIV}} should be {{ic|0}}
|''Random Number'' and ''Encrypted Diversifier'' should be {{ic|0}}.
| {{-}}
|-
|rowspan="6"|
* Logitech MX Anywhere 2S
|
* Copy {{ic|IRK}}.
* Remove the spaces between the hex octets.
| {{Grey|?}}
| {{ic|IdentityResolvingKey.Key}}
|-
|
* Copy {{ic|CSRK}}.
* Remove the spaces between the hex octets.
| {{Grey|?}}
| {{ic|LocalSignatureKey.Key}}
|-
|
* Copy {{ic|LTK}}.
* Remove the spaces between the hex octets.
| {{Grey|?}}
| {{ic|LongTermKey.Key}}
|-
|
* Copy {{ic|KeyLength}}.
* Convert the whole number to decimal.
| {{Grey|?}}
| {{ic|LongTermKey.EncSize}}
|-
|
* Copy {{ic|EDIV}}.
* Convert the whole number to decimal.
| {{Grey|?}}
| {{ic|LongTermKey.EDiv}}
|-
|
* Copy {{ic|ERand}}.
* Reverse the order of the octets.
* Convert the whole number to decimal.
| {{Grey|?}}
| {{ic|LongTermKey.Rand}}
|-
|rowspan="1"|
* Royal Kludge F68 keyboard is like the Logitech MX Anywhere 2S
|
* Also copy {{ic|CSRKInbound}} too.
* Remove the spaces between the hex octets.
| {{Grey|?}}
| {{ic|RemoteSignatureKey.Key}}
|-
|rowspan="4"|
* ThinkPad TrackPoint Keyboard II
* Pebble M350 mouse
* Logitech G604 Lightspeed mouse
|
* Copy {{ic|IRK}}.
* Reverse the order of the octets.
|
* Copy ''Remote IRK''.
* Convert from base64 to hex.
| {{ic|IdentityResolvingKey.Key}}
|-
|
* Copy {{ic|LTK}}.
* Remove the spaces between the hexadecimal octets.
|
* Copy ''Remote Encryption'' > ''Long-term Key''.
* Convert from base64 to hex.
| {{ic|LongTermKey.Key}}
|-
|
* Copy {{ic|ERand}}.
* Reverse the order of the octets.
* Convert the whole number to decimal.
|
* Copy ''Remote Encryption'' > ''Random Number''.
* Convert from base64 to a little-endian decimal number (see Python code below).
| {{ic|LongTermKey.Rand}}
|-
|
* Copy {{ic|EDIV}}.
* Reverse the order of the octets.
* Convert the whole number to decimal.
|
* Copy ''Remote Encryption'' > ''Encrypted Diversifier''.
* Convert from base64 to a little-endian decimal number (see Python code below).
| {{ic|LongTermKey.EDiv}}
|-
|rowspan="3"|Other devices
|
* Copy {{ic|LTK}}.
* Remove the spaces between the hex octets.
|
* Copy ''Remote IRK''.
* Convert from base64 to hex.
| {{ic|LongTermKey.Key}}
|-
|
* Copy {{ic|ERand}}.
* Reverse the order of the octets.
* Convert the whole number to decimal.
|
* Copy ''Remote Encryption'' > ''Long-term Key''.
* Convert from base64 to hex.
| {{ic|LongTermKey.Rand}}
|-
|
* Copy {{ic|EDIV}}.
* Remove the spaces between the hex octets.
|
* Copy ''Remote Encryption'' > ''Encrypted Diversifier''.
* Convert from base64 to hex.
* Reverse the order of the octets.
| {{ic|LongTermKey.EDiv}}
|-
|rowspan="1"|Xbox wireless controller
|
* Copy {{ic|LTK}}.
* Remove the spaces between the hex octets.
| {{Grey|?}}
| {{ic|SlaveLongTermKey.Key}}
|}

{{Note|
* To just remove the spaces from a value, you can use [https://www.browserling.com/tools/remove-all-whitespace this online tool] or this [[Python]] code:
 >>> "''key_value''".replace(" ", "")
* This Python code does only the octet reversal:
{{bc|1=
>>> ERand=" 63 02 84 B8 5D 40 44 DF   "
>>> ERand=list(reversed(ERand.strip().split()))
}}
* This Python code does the additional decimal conversion required for some:
{{bc|1=
>>> int("".join(ERand), 16)
16088054540146049635
}}
* This Python code does the base64 to hex conversion:
 binascii.hexlify(base64.decodebytes(b'...')).upper()
* This Python code does the full macOS Encrypted Diversifier conversion:
 struct.unpack('<H', base64.decodebytes(b'...'))
* This Python code does the full macOS Random Number conversion:
 struct.unpack('&lt;Q', base64.decodebytes(b'...'))
}}

For an example of the general case:
* An {{ic|LTK}} of {{ic|48 4D AF CD 0F 92 22 88 0A 52 9A F4 76 DA 8B 94}} makes for a {{ic|LongTermKey.Key}} of {{ic|484DAFCD0F9222880A529AF476DA8B94}}.
* An {{ic|ERand}} of {{ic|63 02 84 B8 5D 40 44 DF}} makes for a {{ic|Rand}} of {{ic|16088054540146049635}}.
* An {{ic|EDIV}} of {{ic|37520}} makes for an {{ic|EDiv}} of {{ic|37520}}.

==== Saving the configuration ====

Now that you have the keys change user to root, then continue with:

 # cd /var/lib/bluetooth/''BT-Adapter-MAC-address''

Here you will find folders for each paired Bluetooth device. For each device you want to pair with Arch and your dual boot, do the following:

 # cd ''device-MAC-address''

{{Note|At this point, if you are using a device which increments its MAC address on pairing, you must move the MAC address directory to the incremented path. Either copy the MAC address from Windows, or increment it yourself while minding the fact that each octet is a two-digit [[Wikipedia:Hexadecimal|hexadecimal]] number.}}

If you have a pairing key (i.e. this is not a Bluetooth 5.1 device), then edit the {{ic|info}} file and change the key under {{ic|[LinkKey]}}. E.g.:

{{hc|1=info|2=
[LinkKey]
Key=XXXXXXXXXXXXXXX
}}

{{Note|You will have to make sure that all the letters are in capital case. Remove any spaces.}}

If you have several keys, as in Bluetooth 5.1, edit the {{ic|info}} file and substitute all applicable keys with their recorded values. E.g. for an Xbox One S Wireless Controller:

{{hc|1=info|2=
[IdentityResolvingKey]
Key=<IdentityResolvingKey.Key>

[PeripheralLongTermKey]
Key=<PeripheralLongTermKey.Key>

[SlaveLongTermKey]
Key=<SlaveLongTermKey.Key>
}}

Then [[restart]] {{ic|bluetooth.service}} and {{ic|pulseaudio}} (with {{ic|pulseaudio -k && pulseaudio --start}}).

You should be able to connect to your device now.

{{Note|Depending on your Bluetooth manager, you may need to perform a full reboot in order to reconnect to the device.}}
