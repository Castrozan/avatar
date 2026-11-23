from barge_in_listening import (
    NOISE,
    QUIET_SPEECH,
    SILENCE,
    SPEECH,
    TRAILING_SILENCE_FRAMES,
    HeardReply,
)
from barge_in import decision

TWO_SECONDS_OF_FRAMES = 63
COUGH_FRAMES = decision.SPEECH_FRAMES - 1


def test_speech_while_nothing_is_spoken_sends_nothing_and_runs_no_model(tmp_path):
    heard = HeardReply(tmp_path, speaking=False)
    assert heard.hear([SPEECH] * TWO_SECONDS_OF_FRAMES + [SILENCE] * 30) == []
    assert heard.voice_activity.judged == 0


def test_a_long_loud_sound_without_speech_lets_the_reply_finish(tmp_path):
    heard = HeardReply(tmp_path)
    assert heard.hear([NOISE] * TWO_SECONDS_OF_FRAMES) == []


def test_a_cough_shorter_than_the_rule_lets_the_reply_finish(tmp_path):
    heard = HeardReply(tmp_path)
    assert heard.hear([SPEECH] * COUGH_FRAMES + [NOISE] * 40) == []


def test_coughs_in_separate_moments_never_add_up_to_a_barge_in(tmp_path):
    heard = HeardReply(tmp_path)
    cough = [SPEECH] * COUGH_FRAMES + [SILENCE] * TRAILING_SILENCE_FRAMES
    assert heard.hear(cough * 10) == []


def test_the_avatars_quiet_echo_below_the_speech_level_lets_the_reply_finish(
    tmp_path,
):
    heard = HeardReply(tmp_path)
    assert heard.hear([QUIET_SPEECH] * TWO_SECONDS_OF_FRAMES) == []


def test_each_reply_starts_the_voice_model_from_a_fresh_state(tmp_path):
    heard = HeardReply(tmp_path)
    heard.hear([SILENCE] * 3)
    heard.reply.is_speaking = False
    heard.hear([SILENCE] * 3)
    heard.reply.is_speaking = True
    heard.hear([SILENCE] * 3)
    assert heard.voice_activity.resets == 2
