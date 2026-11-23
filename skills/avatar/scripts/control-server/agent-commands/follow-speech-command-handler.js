function handleFollowSpeechCommand(_command, connection, context) {
  context.speechFollowers.follow(connection, context.speechFloor.isSpeaking());
}

module.exports = { handleFollowSpeechCommand };
