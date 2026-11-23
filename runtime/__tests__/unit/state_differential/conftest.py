import os

import pytest
from rust_floor import RustFloor


@pytest.fixture(scope="session")
def rust():
    binary = os.environ.get("AVATAR_FLOOR_BINARY")
    if not binary:
        pytest.skip("AVATAR_FLOOR_BINARY names the Rust avatar-floor binary")
    floor = RustFloor(binary)
    yield floor
    floor.close()
