// TypeScript file with a long function (30+ lines) to test truncation logic

function processData(input: string): number {
    // Line 2
    const step1 = input.trim();

    // Line 5
    const step2 = step1.toLowerCase();

    // Line 8
    const step3 = step2.split(' ');

    // Line 11
    const step4 = step3.filter(word => word.length > 0);

    // Line 14
    const step5 = step4.map(word => word.charAt(0));

    // Line 17: TYPE ERROR HERE
    const errorHere: string = 12345; // Error: number assigned to string

    // Line 20
    const step6 = step5.join('');

    // Line 23
    const step7 = step6.toUpperCase();

    // Line 26
    const step8 = step7.length;

    // Line 29
    return step8;
    // Line 31
}
