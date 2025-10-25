@0xe3a9c3d4089f91da;

interface MessageStream {
    struct MessageContent {
     content @0 :Text;
    }

    read @0 (request: MessageContent) -> (chunk: Data);

}
