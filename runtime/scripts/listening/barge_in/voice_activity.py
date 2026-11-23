from speech_segmenter import FRAME_SECONDS, SAMPLE_RATE

MODEL_ENVIRONMENT_VARIABLE = "AVATAR_VOICE_ACTIVITY_MODEL"
FRAME_SAMPLES = int(SAMPLE_RATE * FRAME_SECONDS)
CONTEXT_SAMPLES = 64
STATE_SHAPE = (2, 1, 128)
FULL_SCALE = 32768


class SileroVoiceActivity:
    def __init__(self, model_path):
        import numpy
        import onnxruntime

        options = onnxruntime.SessionOptions()
        options.execution_mode = onnxruntime.ExecutionMode.ORT_SEQUENTIAL
        options.inter_op_num_threads = 1
        options.intra_op_num_threads = 1
        self.numpy = numpy
        self.session = onnxruntime.InferenceSession(model_path, sess_options=options)
        self.rate = numpy.array(SAMPLE_RATE, dtype=numpy.int64)
        self.reset()

    def reset(self):
        self.state = self.numpy.zeros(STATE_SHAPE, dtype=self.numpy.float32)
        self.context = self.numpy.zeros(CONTEXT_SAMPLES, dtype=self.numpy.float32)

    def window(self, frame):
        samples = self.numpy.frombuffer(frame, dtype=self.numpy.int16)
        audio = samples.astype(self.numpy.float32) / FULL_SCALE
        return self.numpy.pad(audio, (0, FRAME_SAMPLES - len(audio)))

    def speech_probability(self, frame):
        audio = self.window(frame)
        model_input = self.numpy.concatenate([self.context, audio])[None, :]
        output, self.state = self.session.run(
            None, {"input": model_input, "state": self.state, "sr": self.rate}
        )
        self.context = audio[-CONTEXT_SAMPLES:]
        return output[0][0].item()
