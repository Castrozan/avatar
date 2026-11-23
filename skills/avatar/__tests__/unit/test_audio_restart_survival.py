from control_server_node_runner import evaluate_node_json, requires_node


@requires_node
def test_a_missing_avatar_sink_never_sends_the_reply_to_the_speakers_twice():
    result = evaluate_node_json(
        """
        const childProcess = require("child_process");
        const played = [];
        childProcess.spawn = (_, playerArguments) => {
          played.push({
            sink: playerArguments[playerArguments.indexOf("--device") + 1],
            fallsBack: !playerArguments.includes("--property=node.dont-fallback=true"),
          });
          return {
            stderr: { on: () => {} },
            on: (event, resolve) => event === "close" && resolve(),
            kill: () => {},
          };
        };
        const { handleSpeakCommand } = require("./agent-commands/speak-command-handler");
        const context = {
          clientRegistry: { hasRenderer: () => false, sendToRenderer: () => {} },
          textToSpeechGenerator: { generate: async () => ({ audioPath: "/a.wav", source: "local" }) },
          speakerSinkName: "default",
          speechFloor: {
            enqueue: async (_, perform) => {
              await (await perform(() => true)).finished;
              return { status: "finished" };
            },
          },
        };
        const result = handleSpeakCommand(
          { text: "oi", output: "both", id: "turn" },
          { sendJson: () => {} },
          context,
        ).then(() => played);
        """
    )
    assert result == [
        {"sink": "AvatarLipSync", "fallsBack": False},
        {"sink": "AvatarMic", "fallsBack": False},
        {"sink": "default", "fallsBack": True},
    ]


@requires_node
def test_the_lip_sync_meter_listens_again_after_its_recorder_ends_until_stopped():
    result = evaluate_node_json(
        """
        const childProcess = require("node:child_process");
        const { EventEmitter } = require("node:events");
        const recorders = [];
        childProcess.spawn = () => {
          const recorder = new EventEmitter();
          recorder.stdout = new EventEmitter();
          recorder.stderr = new EventEmitter();
          recorder.kill = () => recorder.emit("close");
          recorders.push(recorder);
          return recorder;
        };
        const { startMicrophoneLevelMeter } = require("./audio-playback/microphone-level-meter");
        const levels = [];
        const registry = { sendVoiceLevel: (level) => levels.push(level) };
        const after = (milliseconds) => new Promise((resolve) => setTimeout(resolve, milliseconds));
        const stop = startMicrophoneLevelMeter(registry, "AvatarLipSync.monitor", 5);
        const result = (async () => {
          recorders[0].emit("close");
          await after(40);
          const afterServerRestart = recorders.length;
          const loud = Buffer.alloc(8);
          loud.writeFloatLE(0.5, 0);
          loud.writeFloatLE(-0.5, 4);
          recorders[1].stdout.emit("data", loud);
          await after(50);
          stop();
          await after(40);
          return {
            afterServerRestart,
            afterStop: recorders.length,
            heardOnTheSecondRecorder: levels.includes(0.5),
            endsSilent: levels.at(-1) === 0,
          };
        })();
        """
    )
    assert result == {
        "afterServerRestart": 2,
        "afterStop": 2,
        "heardOnTheSecondRecorder": True,
        "endsSilent": True,
    }
