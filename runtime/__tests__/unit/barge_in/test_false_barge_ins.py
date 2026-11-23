import barge_in
from barge_in_listening import (
    FRAMES_BEFORE_BARGE_IN,
    LONGEST_SEGMENT_FRAMES,
    hear_sound,
    listening_with,
)

REPLY_AT_TWENTY_ONE_EIGHTEEN = (
    "A skill já mora no repositório do avatar: os dotfiles só a instalam a partir "
    "do flake, junto com as outras."
)
REPLY_AT_TWENTY_ONE_TWENTY_FIVE = (
    "Status. Prontos: a migração pro flake, o plugin no repositório do avatar e a "
    "primeira fatia em Rust."
)


def test_a_long_sound_without_words_lets_the_reply_finish(tmp_path):
    listening = listening_with(tmp_path, "", [REPLY_AT_TWENTY_ONE_EIGHTEEN])
    hear_sound(listening, LONGEST_SEGMENT_FRAMES - 1)
    listening.interrupt_speaker.assert_not_called()
    assert listening.recognize_words.call_count == barge_in.CHECKS_PER_SEGMENT


def test_sound_while_nothing_is_spoken_neither_stops_nor_transcribes(tmp_path):
    listening = listening_with(tmp_path, "Ok, abre eles aí no Chrome", [])
    hear_sound(listening, LONGEST_SEGMENT_FRAMES - 1)
    listening.interrupt_speaker.assert_not_called()
    listening.recognize_words.assert_not_called()


def test_the_avatars_own_voice_leaking_back_lets_the_reply_finish(tmp_path):
    leaked = "a migração pro flake, o plugin no repositório"
    listening = listening_with(tmp_path, leaked, [REPLY_AT_TWENTY_ONE_TWENTY_FIVE])
    hear_sound(listening, LONGEST_SEGMENT_FRAMES - 1)
    listening.interrupt_speaker.assert_not_called()


def test_a_slightly_misheard_echo_still_counts_as_the_avatars_voice(tmp_path):
    misheard = "a migracao pro flak o plugin no repositorio"
    listening = listening_with(tmp_path, misheard, [REPLY_AT_TWENTY_ONE_TWENTY_FIVE])
    hear_sound(listening, LONGEST_SEGMENT_FRAMES - 1)
    listening.interrupt_speaker.assert_not_called()


def test_a_sound_too_short_to_be_checked_is_never_sent_to_the_recognizer(tmp_path):
    listening = listening_with(tmp_path, "Para", [REPLY_AT_TWENTY_ONE_EIGHTEEN])
    hear_sound(listening, FRAMES_BEFORE_BARGE_IN - 1)
    listening.recognize_words.assert_not_called()
    listening.interrupt_speaker.assert_not_called()


def test_a_recognizer_that_fails_lets_the_reply_finish(tmp_path, capsys):
    listening = listening_with(tmp_path, "", [REPLY_AT_TWENTY_ONE_EIGHTEEN])
    listening.recognize_words.side_effect = ConnectionError("Hex is not running")
    hear_sound(listening, LONGEST_SEGMENT_FRAMES - 1)
    listening.interrupt_speaker.assert_not_called()
    assert "Hex is not running" in capsys.readouterr().err
