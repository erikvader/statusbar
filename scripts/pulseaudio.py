#!/bin/python

import dbus
from dbus.mainloop.glib import DBusGMainLoop
from gi.repository import GLib
import os

def connect():
    if 'PULSE_DBUS_SERVER' in os.environ:
        address = os.environ['PULSE_DBUS_SERVER']
    else:
        bus = dbus.SessionBus()
        server_lookup = bus.get_object("org.PulseAudio1", "/org/pulseaudio/server_lookup1")
        address = server_lookup.Get("org.PulseAudio.ServerLookup1", "Address", dbus_interface="org.freedesktop.DBus.Properties")

    return dbus.connection.Connection(address)

def find_sink(conn):
    sink_obj = conn.get_object("org.pulseaudio.Server", "/org/pulseaudio/core1")
    sinks = sink_obj.Get("org.PulseAudio.Core1", "Sinks", dbus_interface="org.freedesktop.DBus.Properties")
    # NOTE: assuming that there is one and only one
    sink = next(iter(sinks))
    return sink

def is_muted(sink_obj):
    mute = sink_obj.Get("org.PulseAudio.Core1.Device", "Mute", dbus_interface="org.freedesktop.DBus.Properties")
    return mute == 1

def volume_perc(raw_vols):
    # NOTE: assuming all channels has the same volume
    chan1 = next(iter(raw_vols))
    return round((chan1 / 65537) * 100)

def get_volume(sink_obj):
    volumes = sink_obj.Get("org.PulseAudio.Core1.Device", "Volume", dbus_interface="org.freedesktop.DBus.Properties")
    return volume_perc(volumes)

def main():
    DBusGMainLoop(set_as_default=True)
    conn = connect()
    sink = find_sink(conn)
    sink_obj = conn.get_object("org.pulseaudio.Server", sink)

    volume = get_volume(sink_obj)
    muted = is_muted(sink_obj)

    def print_dzen():
        if muted:
            print("^fg(gray)muted^fg()", flush=True)
        else:
            print("{}%".format(volume), flush=True)

    print_dzen()

    core = conn.get_object("org.pulseaudio.Server", "/org/pulseaudio/core1")
    core.ListenForSignal("org.PulseAudio.Core1.Device.VolumeUpdated", [sink], dbus_interface="org.PulseAudio.Core1")
    core.ListenForSignal("org.PulseAudio.Core1.Device.MuteUpdated", [sink], dbus_interface="org.PulseAudio.Core1")

    def on_volume_change(volumes):
        nonlocal volume
        volume = volume_perc(volumes)
        print_dzen()

    def on_mute_change(mute):
        nonlocal muted
        muted = mute == 1
        print_dzen()

    conn.add_signal_receiver(on_volume_change, signal_name="VolumeUpdated")
    conn.add_signal_receiver(on_mute_change, signal_name="MuteUpdated")

    loop = GLib.MainLoop()
    loop.run()

if __name__ == "__main__":
    main()
