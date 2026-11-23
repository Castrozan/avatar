from control_server_node_runner import evaluate_node_json, requires_node
from reply_playback_harness import REPLY_PLAYBACK_HARNESS

NO_RENDERER_CONNECTED = (
    REPLY_PLAYBACK_HARNESS
    + """
const {
  ConnectedClientRegistry,
} = require("./avatar-session/connected-client-registry");
context.clientRegistry = new ConnectedClientRegistry();
const sentence = (word) => `${word} ${"and then some more words ".repeat(6)}end. `;
const reply = ["One", "Two", "Three"].map(sentence).join("");
const acknowledgements = () =>
  agentMessages
    .filter((message) => message.type !== "stopSpeakingAck")
    .map(({ type, status, error }) => ({ type, status, error: error ?? null }));
"""
)


@requires_node
def test_a_chunked_reply_plays_every_sentence_and_finishes_with_no_renderer():
    result = evaluate_node_json(
        NO_RENDERER_CONNECTED
        + """
        const result = (async () => {
          const spoken = speak(reply);
          for (let sentence = 0; sentence < 3; sentence += 1) await finishRoomPlayback();
          await spoken;
          return {
            played: roomPlayers().map((player) => player.audio),
            acknowledgements: acknowledgements(),
            rendererConnected: context.clientRegistry.connectionSummary().rendererConnected,
          };
        })();
        """
    )
    assert result == {
        "played": ["/One.wav", "/Two.wav", "/Three.wav"],
        "acknowledgements": [{"type": "speakAck", "status": "finished", "error": None}],
        "rendererConnected": False,
    }


@requires_node
def test_a_barge_in_interrupts_the_reply_with_no_renderer():
    result = evaluate_node_json(
        NO_RENDERER_CONNECTED
        + """
        const result = (async () => {
          const spoken = speak(reply);
          await settle();
          bargeIn();
          await spoken;
          return {
            played: roomPlayers().map((player) => [player.audio, player.signals]),
            acknowledgements: acknowledgements(),
          };
        })();
        """
    )
    assert result == {
        "played": [["/One.wav", ["SIGTERM"]]],
        "acknowledgements": [
            {"type": "speakAck", "status": "interrupted", "error": None}
        ],
    }


@requires_node
def test_the_live_voice_mouth_level_is_accepted_with_no_renderer():
    result = evaluate_node_json(
        NO_RENDERER_CONNECTED
        + """
        const { handleVoiceLevelCommand } = require("./agent-commands/voice-level-command-handler");
        handleVoiceLevelCommand({ value: 0.5 }, connection, context);
        const result = { replies: agentMessages };
        """
    )
    assert result == {"replies": []}
