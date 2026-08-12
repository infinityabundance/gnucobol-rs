
       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.
       DATA  DIVISION.
       WORKING-STORAGE SECTION.
   *> D.22.7.1    Example of validation of USAGE DISPLAY items
   *>
   *>*************************************************************
   *>Description of target record (note: should be possible to define later)
   *>*************************************************************
   *>This is set up by the optional DESTINATION clauses defined
   *>in the input record;
   *>if a format error is found, a default value is stored instead.
    01  TARGET-AREA.
        05  OUT-NAME       PIC X(20).
        05  OUT-WEEK       PIC 99 COMP   OCCURS 5.
   *>*************************************************************
   *>Validated items
   *>*************************************************************
    01  INPUT-RECORD.
   *>PIC 99 checks that IN-TYPE is 2 characters numeric;
        03  IN-TYPE         PIC 99
   *>if IN-TYPE fails the PICTURE check, it is assumed to be 1;
   *>without a DEFAULT clause, the assumed value would here be 0.
                                DEFAULT 1.
   *>PRESENT WHEN states the condition for this format to be used.
        03  IN-REC-FORMAT-1 PRESENT WHEN IN-TYPE = 0 OR 1 OR 2.
   *>PICTURE A(20) checks for 20 alphabetic (or space) characters.
            05  IN-NAME       PIC A(20)
   *>PRESENT WHEN defines when the validation clauses for this data item apply:
                                 PRESENT WHEN IN-TYPE = 0 OR 1
   *>CLASS checks each character for a class defined in SPECIAL-NAMES
   *>or a predefined class
                                 CLASS IS ALPHABETIC-UPPER
   *>DESTINATION moves this item (or spaces if not alpha) to OUT-NAME.
                                 DESTINATION OUT-NAME.
   *>PRESENT WHEN checks whether the item is "blank" under this condition
            05  FILLER REDEFINES IN-NAME  PIC X(20) *> should PIC be optional?
                                 PRESENT WHEN IN-TYPE = 2
                                 DESTINATION OUT-NAME.
                88               VALUE SPACES IS VALID.
   *>The values of IN-WEEK are checked to be in non-descending order.
            05  IN-WEEK       PIC 99  OCCURS 5
                VARYING IN-WEEK-NO FROM 1, IN-NEXT-WEEK-NO FROM 2
                INVALID WHEN IN-WEEK-NO < 5
                AND IN-WEEK (IN-WEEK-NO) > IN-WEEK (IN-NEXT-WEEK-NO)
   *>OUT-WEEK (1) to (5) will hold the values of IN-WEEK (1) to (5),
   *>or zero for any one that failed the format (PICTURE) test.
                DESTINATION OUT-WEEK (IN-WEEK-NO).
   *>The 88-level INVALID entries check for invalid ranges of values.
               88  VALUES 0, 53 THRU 99 ARE INVALID.
   *>REDEFINES and another PRESENT WHEN define an alternate format.
        03  IN-REC-FORMAT-2 REDEFINES IN-REC-FORMAT-1
                                        PRESENT WHEN IN-TYPE > 2.
   *>IN-PAY has insertion characters that must be present on input.
            05  IN-PAY                PIC ZZ,ZZZ.ZZ.
   *>The 88-level VALID entries check for valid ranges of values;
   *>the condition-name, if present, may be used in the usual way.
   *>The following assume that DECIMAL POINT IS COMMA is not specified.
                 88  IN-PAY-OK  VALUES "10,000.00" THRU "20,000.00" ARE VALID.
   *>88-level entries may also have a condition attached.
                 88                    VALUES "20,000.01" THRU "30,000.00" ARE VALID
                                       WHEN IN-TYPE = 8.
   *>exceptional cases can be specified using PRESENT WHEN
            05  IN-CODE                PIC AX(3)9(4)
                                       PRESENT WHEN IN-CODE NOT = "UNKNOWN".
            05  FILLER           PIC X(13).
   *>
   *>*************************************************************
   *> Description of error messages
   *>*************************************************************
   *>Error messages or flags are set up or cleared automatically
   *>when the VALIDATE statement is executed; the programmer chooses
   *>where they go and what messages or values they contain;
   *>they need not be contiguous as they are in this example.
   01  VALIDATE-MESSAGES.
       03  PIC X(40)  VALIDATE-STATUS "Unknown Record Type - 1 assumed"
                        WHEN ERROR FOR IN-TYPE
   *> more than one VALIDATE-STATUS clause may be defined in one entry;
   *> a NO ERROR  phrase produces a message when the item is valid.
                      VALIDATE-STATUS "Record type Accepted"
                        WHEN NO ERROR FOR IN-TYPE.
   *> The VALIDATE-STATUS clause can pinpoint the stage of the failed check.
       03  PIC X(40)  VALIDATE-STATUS "Name not alphabetic"
                        WHEN ERROR ON FORMAT FOR IN-NAME
                      VALIDATE-STATUS "Lower-case not allowed in name"
                        WHEN ERROR ON CONTENT FOR IN-NAME
                      VALIDATE-STATUS "Name not allowed in this case"
                        WHEN ERROR ON RELATION FOR IN-NAME.
   *> If no message is stored, spaces will be stored in these cases.
   *> Errors may also be indicated by flags;
   *> they may also refer to a table of input items.
       03  W-ERROR-FLAG   PIC 9   COMP  OCCURS 5
                      VALIDATE-STATUS 1 WHEN ERROR FOR IN-WEEK.
   *>An EC-VALIDATE (nonfatal) exception is also set if the
   *>VALIDATE statement detects an invalid condition.


   *> From D.22.7.2    Example of validation of non-display items:
      01  MIXED-GROUP TYPEDEF STRONG.
          05  FLD-1             PIC S9(4)  USAGE COMP.
          05  FLD-2             PIC S9(7)  USAGE PACKED-DECIMAL.
          05  FLD-3             PIC 1(8)   USAGE BIT ALIGNED.
          05  PTR-1             USAGE INDEX.
          05  PTR-2             USAGE OBJECT REFERENCE.
          05  TXT-1             PIC N(12)  USAGE NATIONAL.
    01 MY-MIXED-GROUP TYPE MIXED-GROUP.


   *>
   *>*************************************************************
   *>Execution of the VALIDATE statement
   *>*************************************************************
    PROCEDURE DIVISION.

   *>A single VALIDATE statement performs all the actions implied
   *>in the above data descriptions.
        VALIDATE INPUT-RECORD
   *>After this statement has been executed:
   *>(1) the input record is unchanged;
   *>(2) input items are moved automatically to the target area;
   *>(3) error messages are set up wherever specified in the program.
   *>

   *> From D.22.7.2    Example of validation of non-display items:
   *>A declarative section could be used instead of VALIDATE-STATUS clauses
   *>especially if errors are not expected.
   *>> TURN EC-VALIDATE CHECKING ON
        VALIDATE MY-MIXED-GROUP

     GOBACK.
