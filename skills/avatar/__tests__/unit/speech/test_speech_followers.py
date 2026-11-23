from control_server_node_runner import evaluate_node_json, requires_node

FOLLOWERS_HARNESS = """
const { SpeechFloor } = require("./speech/speech-floor");
const { SpeechFollowers } = require("./avatar-session/speech-followers");
const { routeAgentCommand } = require("./agent-commands/agent-command-router");
const followers = new SpeechFollowers();
const floor = new SpeechFloor({ async open() { return null; } }, (speaking) =>
  followers.announce(speaking),
);
const context = { speechFloor: floor, speechFollowers: followers };
const connection = () => {
  const closeHandlers = [];
  return {
    sent: [],
    sendJson(payload) { this.sent.push(payload); return true; },
    onClosed(handle) { closeHandlers.push(handle); },
    close() { closeHandlers.forEach((handle) => handle()); },
  };
};
const reply = () => {
  let finish;
  const finished = new Promise((resolve) => { finish = resolve; });
  return {
    finish,
    perform: async () => ({ finished, stop: () => finish() }),
  };
};
const settle = () => new Promise((resolve) => setTimeout(resolve, 5));
const activity = (follower) =>
  follower.sent.filter((payload) => payload.type === "speechActivity")
    .map((payload) => payload.speaking);
"""


@requires_node
def test_a_follower_hears_when_replies_start_and_when_the_last_one_ends():
    result = evaluate_node_json(
        FOLLOWERS_HARNESS
        + """
        const follower = connection();
        const result = (async () => {
          await routeAgentCommand('{"type":"followSpeech"}', follower, context);
          const first = reply();
          const second = reply();
          const firstDone = floor.enqueue(false, first.perform);
          const secondDone = floor.enqueue(false, second.perform);
          await settle();
          first.finish();
          await firstDone;
          const afterFirst = activity(follower);
          second.finish();
          await secondDone;
          return { afterFirst, all: activity(follower) };
        })();
        """
    )
    assert result["afterFirst"] == [False, True]
    assert result["all"] == [False, True, False]


@requires_node
def test_a_follower_joining_mid_reply_learns_the_avatar_is_speaking():
    result = evaluate_node_json(
        FOLLOWERS_HARNESS
        + """
        const follower = connection();
        const spoken = reply();
        const result = (async () => {
          const done = floor.enqueue(false, spoken.perform);
          await settle();
          await routeAgentCommand('{"type":"followSpeech"}', follower, context);
          spoken.finish();
          await done;
          return activity(follower);
        })();
        """
    )
    assert result == [True, False]


@requires_node
def test_a_stop_from_the_follower_interrupts_the_reply_and_ends_the_activity():
    result = evaluate_node_json(
        FOLLOWERS_HARNESS
        + """
        const follower = connection();
        const result = (async () => {
          await routeAgentCommand('{"type":"followSpeech"}', follower, context);
          const done = floor.enqueue(false, reply().perform);
          await settle();
          await routeAgentCommand('{"type":"stopSpeaking"}', follower, context);
          const { status } = await done;
          return { status, activity: activity(follower) };
        })();
        """
    )
    assert result == {"status": "interrupted", "activity": [False, True, False]}


@requires_node
def test_a_closed_follower_hears_nothing_more():
    result = evaluate_node_json(
        FOLLOWERS_HARNESS
        + """
        const follower = connection();
        const result = (async () => {
          await routeAgentCommand('{"type":"followSpeech"}', follower, context);
          follower.close();
          const spoken = reply();
          const done = floor.enqueue(false, spoken.perform);
          spoken.finish();
          await done;
          return activity(follower);
        })();
        """
    )
    assert result == [False]
