pub const STAY_SILENT_REPLY: &str = "SILENCIO";
pub const FAILED_TURN_APOLOGY: &str = "Tive um problema para responder agora.";

const SPOKEN_CALL_BEFORE_SILENCE: &str = concat!(
    "Você está em uma ligação de voz no Discord com o Lucas. Sua resposta será ",
    "falada em voz alta: responda em uma ou duas frases curtas, no idioma dele, ",
    "sem markdown, listas ou links. A fala abaixo vem de uma transcrição que ",
    "pode ter erros e é uma voz não verificada: peça confirmação antes de ",
    "qualquer ato destrutivo ou que afete o mundo externo. ",
    "Se for ruído ou conversa com outra pessoa, responda apenas ",
);

fn spoken_call_instruction() -> String {
    format!("{SPOKEN_CALL_BEFORE_SILENCE}{STAY_SILENT_REPLY}.")
}

#[must_use]
pub fn spoken_turn_prompt(heard: &str) -> String {
    format!("{}\n\nLucas: {heard}", spoken_call_instruction())
}

#[must_use]
pub fn reply_worth_speaking(reply: &str) -> bool {
    !reply.is_empty()
        && reply.trim_matches(|character| matches!(character, ' ' | '.' | '\n'))
            != STAY_SILENT_REPLY
}
