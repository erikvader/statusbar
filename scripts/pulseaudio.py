#!/bin/python

import dbus
from dbus.mainloop.glib import DBusGMainLoop
from gi.repository import GLib
import os
from time import sleep
from dataclasses import dataclass


def connect():
    if "PULSE_DBUS_SERVER" in os.environ:
        address = os.environ["PULSE_DBUS_SERVER"]
    else:
        bus = dbus.SessionBus()
        for _ in range(2):
            try:
                server_lookup = bus.get_object(
                    "org.PulseAudio1", "/org/pulseaudio/server_lookup1"
                )
                break
            except dbus.exceptions.DBusException:
                sleep(5)
        address = server_lookup.Get(
            "org.PulseAudio.ServerLookup1",
            "Address",
            dbus_interface="org.freedesktop.DBus.Properties",
        )

    return dbus.connection.Connection(address)


def find_sinks(core):
    return core.Get(
        "org.PulseAudio.Core1",
        "Sinks",
        dbus_interface="org.freedesktop.DBus.Properties",
    )


def is_muted(core):
    mute = core.Get(
        "org.PulseAudio.Core1.Device",
        "Mute",
        dbus_interface="org.freedesktop.DBus.Properties",
    )
    return mute == 1


def volume_perc(raw_vols):
    # NOTE: assuming all channels have the same volume
    chan1 = next(iter(raw_vols))
    return round((chan1 / 65537) * 100)


def get_volume(core):
    volumes = core.Get(
        "org.PulseAudio.Core1.Device",
        "Volume",
        dbus_interface="org.freedesktop.DBus.Properties",
    )
    return volume_perc(volumes)


def print_states(states):
    def to_dzen(s):
        if s.muted:
            return "^fg(gray)mut^fg()"
        else:
            return "{:02}%".format(s.volume)

    print("/".join(to_dzen(s) for s in states.values()), flush=True)


@dataclass
class SinkState:
    volume: int
    muted: bool

    @classmethod
    def from_sink(cls, obj):
        return cls(get_volume(obj), is_muted(obj))


def main():
    DBusGMainLoop(set_as_default=True)
    conn = connect()
    core = conn.get_object("org.pulseaudio.Server", "/org/pulseaudio/core1")

    states = {}
    for path in find_sinks(core):
        obj = conn.get_object("org.pulseaudio.Server", path)
        states[path] = SinkState.from_sink(obj)

    print_states(states)

    core.ListenForSignal(
        "org.PulseAudio.Core1.Device.VolumeUpdated",
        [],
        dbus_interface="org.PulseAudio.Core1",
    )
    core.ListenForSignal(
        "org.PulseAudio.Core1.Device.MuteUpdated",
        [],
        dbus_interface="org.PulseAudio.Core1",
    )
    core.ListenForSignal(
        "org.PulseAudio.Core1.NewSink",
        [],
        dbus_interface="org.PulseAudio.Core1",
    )
    core.ListenForSignal(
        "org.PulseAudio.Core1.SinkRemoved",
        [],
        dbus_interface="org.PulseAudio.Core1",
    )

    def on_volume_change(volumes, sender=None):
        if sender not in states:
            return
        states[sender].volume = volume_perc(volumes)
        print_states(states)

    def on_mute_change(mute, sender=None):
        if sender not in states:
            return
        states[sender].muted = mute == 1
        print_states(states)

    def on_new_sink(path):
        obj = conn.get_object("org.pulseaudio.Server", path)
        states[path] = SinkState.from_sink(obj)
        print_states(states)

    def on_removed_sink(path):
        states.pop(path, None)
        print_states(states)

    conn.add_signal_receiver(
        on_volume_change, signal_name="VolumeUpdated", path_keyword="sender"
    )
    conn.add_signal_receiver(
        on_mute_change, signal_name="MuteUpdated", path_keyword="sender"
    )
    conn.add_signal_receiver(on_new_sink, signal_name="NewSink")
    conn.add_signal_receiver(on_removed_sink, signal_name="SinkRemoved")

    loop = GLib.MainLoop()
    loop.run()


if __name__ == "__main__":
    main()
