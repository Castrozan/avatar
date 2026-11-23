from control_server_node_runner import evaluate_node_json, requires_node


@requires_node
def test_chrome_request_handles_immediate_replies_and_protocol_errors():
    result = evaluate_node_json(
        """
        const { ChromeDevtoolsSession } = require("./chrome-devtools/chrome-devtools-session");
        let listener;
        const client = {
          onMessage: callback => { listener = callback; return () => {}; },
          sendJson: request => listener(JSON.stringify(request.method === "good" ? {id: request.id, result: {value: 7}} : {id: request.id, error: {message: "unsupported"}})),
        };
        const session = new ChromeDevtoolsSession(client);
        const result = (async () => {
          const good = await session.sendAndAwaitResult("good");
          let error;
          try { await session.sendAndAwaitResult("bad"); } catch (failure) { error = failure.message; }
          return { good, error };
        })();
        """
    )
    assert result == {"good": {"value": 7}, "error": "unsupported"}


@requires_node
def test_chrome_request_timeout_unsubscribes():
    result = evaluate_node_json(
        """
        const { ChromeDevtoolsSession } = require("./chrome-devtools/chrome-devtools-session");
        let removed = false;
        const session = new ChromeDevtoolsSession({ onMessage: () => () => { removed = true; }, sendJson: () => {} });
        const result = session.sendAndAwaitResult("never", {}, 5).catch(error => ({ removed, message: error.message }));
        """
    )
    assert result == {"removed": True, "message": "Chrome did not answer never"}
