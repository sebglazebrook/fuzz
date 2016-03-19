FROM sebglazebrook/rust-nightly

RUN apt-get update && apt-get install -y libncurses5-dev libncursesw5-dev
