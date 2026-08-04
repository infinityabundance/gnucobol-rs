
       identification   division.
       program-id.      prog.
       data             division.
       working-storage  section.
       77 p        usage pointer.
       01 val      pic x(10).
          88 val-i1 value 'some'.
          88 val-i2 value 'val'.
       77 val2     pic x(50).
       77 target   pic x(50).
       01 vnum     pic 9.
          88 vnum-1 value  1.
          88 vnum-2 values 2 thru 5.
          88 vnum-9 value  9.
       01 filler.
          02 tentry pic x occurs 0 to 6 depending on vnum-1.
       procedure        division.
           string   val-i1 delimited by size  into target
           string   val2   delimited by val-i2 into target
           string   val2   delimited by size   into val-i1
                           with pointer val-i2
           unstring val-i1                     into target
           unstring val2   delimited by val-i1 into target
           unstring val2                       into val-i1
                           with pointer val-i2
           move val-i1
             to val-i2, tentry (vnum-9)
           compute vnum-1 = vnum
           compute vnum    = vnum-1 /
                             vnum-2
           go to val-i1
           set p to val-i1
           set p to address of val-i2.
