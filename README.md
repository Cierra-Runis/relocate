## Setup

```sh
rustup override set nightly
```

## Convert Diagram

```mermaid
flowchart TD
    Bytes["Vec&lt;u8&gt;"] --> MIDI["MIDI(Vec&lt;Chunk&gt;)"];

    Bytes --> MIDIFile["MIDIFile(Vec&lt;u8&gt;)"] --> ChunksFile["ChunksFile(Vec&ltChunkFile&gt;)"];

    ChunkFile["ChunkFile"] --> HeaderChunkFile & TrackChunkFile;
    HeaderChunkFile --> HeaderChunk;
    TrackChunkFile --> TrackEventsFile --> TrackChunk["TrackChunk(Vec&lt;TrackEvent&gt;)"];

    TrackEventFile --> TrackEvent;

    ChunksFile ..-> ChunkFile;
    TrackEventsFile ..-> TrackEventFile;
    TrackEvent ..-> TrackChunk;
    TrackChunk & HeaderChunk ..-> Chunk;
    Chunk ..-> MIDI;

    subgraph "File Representation"
        MIDIFile
        ChunksFile
        ChunkFile
        HeaderChunkFile
        TrackChunkFile
        TrackEventsFile
        TrackEventFile
    end

    subgraph "Core Representation"
        MIDI
        Chunk
        HeaderChunk
        TrackChunk
        TrackEvent
    end
```

## Specification

- [Standard MIDI Files 1.0](https://drive.google.com/file/d/1t4jcCCKoi5HMi7YJ6skvZfKcefLhhOgU/view)
- [MIDI Clip File Specification – SMF MIDI 2.0](https://midi.org/midi-clip-file-specification-smf-midi-2-0)
