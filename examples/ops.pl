// math
1 1 +     dump // Sums 1 + 1
3 1 -     dump // Subtracts 3 by 1
2 2 *     dump // Multiplies 2 by 2
1.0 2.0 / dump // Divides 1 by 2 (half)

// comparison
1 1 =  dump // Compares 1 == 1
3 1 != dump // Compares 3 != 1
2 2 >  dump // Compares 2 > 2
2 2 >= dump // Compares 2 >= 2
1 2 <  dump // Compares 1 < 2
1 2 <= dump // Compares 1 <= 2

// stack operations
1     drop                // There's nothing here anymore :(
1     dup  dump dump      // There are two values here now :)
1 2   over dump dump dump // Now it's 1 2 1
1 2 3 rot  dump dump dump // Rotated numbers: 2 3 1 
69 96 swap dump dump      // They even swapped the digits!