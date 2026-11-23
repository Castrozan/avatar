import addressing_decision
from hypothesis import given, settings
from hypothesis import strategies as st
from strategies import SERVANTS, texts

EXAMPLES = settings(max_examples=300, deadline=None)
turns = st.builds(
    lambda speaker, text: {"speaker": speaker, "text": text},
    st.sampled_from((None, "", "Ana", "Lucas")),
    texts,
)
holders = st.sampled_from((None,) + SERVANTS)
candidate_lists = st.lists(st.sampled_from(SERVANTS), max_size=4, unique=True)
confidences = st.one_of(
    st.none(), st.booleans(), st.floats(-0.5, 1.5), st.text(max_size=3)
)
answers = st.lists(
    st.fixed_dictionaries(
        {"type": st.sampled_from(("choice", "text")), "name": st.sampled_from(("addressee", "other"))},
        optional={
            "choice": st.sampled_from(SERVANTS + ("nobody", "holder")),
            "confidence": confidences,
        },
    ),
    max_size=3,
)


def python_turns(items):
    return [addressing_decision.Turn(item["speaker"], item["text"]) for item in items]


@EXAMPLES
@given(texts, st.lists(turns, max_size=4), holders, candidate_lists)
def test_request_matches_python(rust, utterance, items, holder, candidates):
    expected = addressing_decision.request(
        utterance, python_turns(items), holder, candidates
    )
    actual = rust.call(
        "decision_request",
        utterance=utterance,
        turns=items,
        holder=holder,
        candidates=candidates,
    )
    assert (actual["messages"], actual["questions"]) == expected


@EXAMPLES
@given(answers, candidate_lists)
def test_verdict_matches_python(rust, given_answers, candidates):
    expected = addressing_decision.verdict(given_answers, candidates)
    actual = rust.call("decision_verdict", answers=given_answers, candidates=candidates)
    assert actual == {
        "route": expected.route.value,
        "choice": expected.choice,
        "confidence": expected.confidence,
    }
