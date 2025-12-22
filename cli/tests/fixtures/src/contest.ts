// This file is used to test that .contains(".test.") doesn't cause false positives
// The word "contest" contains "test" but this is NOT a test file

export interface Contest {
    id: number;
    name: string;
    participants: string[];
}

export function createContest(name: string): Contest {
    return {
        id: Math.random(),
        name,
        participants: []
    };
}
