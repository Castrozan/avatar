const SPEECH_ACTIVITY = "speechActivity";

class SpeechFollowers {
  constructor() {
    this.connections = new Set();
  }

  follow(connection, speaking) {
    this.connections.add(connection);
    connection.onClosed(() => this.connections.delete(connection));
    connection.sendJson({ type: SPEECH_ACTIVITY, speaking });
  }

  announce(speaking) {
    for (const connection of this.connections)
      connection.sendJson({ type: SPEECH_ACTIVITY, speaking });
  }
}

module.exports = { SpeechFollowers };
