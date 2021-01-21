FROM centos:7.7.1908

RUN yum install -y capnproto gcc g++ python curl git openssl-devel pkg-config gcc-c++ make

RUN curl "https://bootstrap.pypa.io/get-pip.py" -o "get-pip.py" && python get-pip.py

RUN pip install ninja

RUN curl -L https://github.com/Kitware/CMake/releases/download/v3.19.3/cmake-3.19.3.tar.gz > cmake.tar.gz && tar xvf cmake.tar.gz

RUN cd cmake-3.19.3 && ./configure && gmake -j8

RUN cd cmake-3.19.3 && gmake install

RUN git clone https://github.com/rust-lang/rust.git

RUN cd rust && git checkout 1.38.0


RUN cd rust && cp config.toml.example config.toml
RUN cd rust && sed -i 's/#allow-old-toolchain = false/allow-old-toolchain = true/' config.toml
RUN cd rust && ./x.py build
RUN cd rust && ./x.py install
RUN cd rust && ./x.py install cargo
RUN curl -O https://capnproto.org/capnproto-c++-0.6.1.tar.gz && tar xvf capnproto-c++-0.6.1.tar.gz
RUN cd capnproto-c++-0.6.1 && ./configure && make -j6 check
RUN cd capnproto-c++-0.6.1 && make install
