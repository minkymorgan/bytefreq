#!/usr/local/bin/gawk
#
#----------------------------------------------------------------#
#
#             
#               __               _       _    __   
#              / /_  _ __   ___ (_)_ __ | |_ / /_  
#             | '_ \| '_ \ / _ \| | '_ \| __| '_ \ 
#             | (_) | |_) | (_) | | | | | |_| (_) |
#              \___/| .__/ \___/|_|_| |_|\__|\___/ 
#                   |_|                            
#             
#                
#
# Data Strategy | Data Architecture | Data Science & Engineering 
#----------------------------------------------------------------#
#
# ByteFreq_1.0.5 Mask Based Data Profiling
# License: GPLv3
# Author: Andrew Morgan
#
# Instructions for use
# 
# DEPENDECIES:
# This code is in POSIX compliant awk. It should work with any compliant awk interpreter.
# It's tested against awk, gawk, mawk, goawk

# PERFORMANCE:
# It performs fastest with mawk. brew install mawk
# Gawk is recommended, is excellent and maintained. 
# On a mac, "brew install gawk". I suggest you also get gsed. "brew install gsed"
#
# USAGE:
# on the commandline, you call the profiler as a gawk script, here are some examples, which is also the test suite data:
#
#     #GENERATE HUMAN READABLE REPORT - popular eyeball inspection report
#     gawk -F"|" -f bytefreq_v1.05.awk -v header="1" -v report="1" -v grain="L" testdata/UKCompanySample.pip > UkCompanySample.rpt1.txt
# 
#     #GENERATE DATABASE LOADABLE REPORT SUMMARY OUTPUTS - for automation, used for drift in quality analysis 
#     gawk -F"|" -f bytefreq_v1.05.awk -v header="1" -v report="0" -v grain="L" testdata/UKCompanySample.pip > UkCompanySample.rpt0.txt 
#
#     #GENERATE DATABASE LOADABLE RAW+PROFILED DATA - for manual cleansing, find bad datapoints and fix them
#     gawk -F"|" -f bytefreq_v1.05.awk -v header="1" -v report="2" -v grain="L" testdata/UKCompanySample.pip > UkCompanySample.raw2.txt
#
#     #GENERATE DATABASE LOADABLE LONGFORMAT RAW DATA - for automated remediation
#     gawk -F"|" -f bytefreq_v1.05.awk -v header="1" -v report="3" -v grain="L" testdata/UKCompanySample.pip > UkCompanySample.raw3.txt
#
# The options on the command line are: 
#      use standard awk -F option in Awk to set your delimiter for the file
#      use standard awk -v option to set global variables from the command line:
#
#      -v header="0"   indicates your input data has not got a header row.
#      -v header="1"   indicates your input data has a header in row 1, which is recommended.
#
#      -v report="0"   outputs profile frequency reports, loadable as data 
#      -v report="1"   outputs profile frequency reports, human readable and printable 
#      -v report="2"   outputs your raw input data alongside formated profile strings. doubles your columns
#      -v report="3"   outputs your raw + profile data stacked in a long format, suitable for clikhouse reporting
#                      ** note this does not aggregate the data. You get row*column outputs. Aggregate it in a proper database.
#      -v grain="H"    is the option to have granular reports, "L" is the option for simplified profiles
#      -v grain="L"    is the option to have Low Grain reports - best for high 
#      -v awk="awk"    experimental: override to allow using old versions of awk that can't do asort
#                      ** note the override only works well on non-human readable reports
#                       
################################################################################################################# 
################################################################################################################# 
#  I have inlined the quicksort module from runawk below, as asorti is not POXIX compliant or portable
###
#
# Written by Aleksey Cheusov <vle@gmx.net>, public domain
#
# This awk module is a part of RunAWK distribution,
#        http://sourceforge.net/projects/runawk
#
############################################################

# =head2 quicksort.awk
#
# =over 2
#
# =item I<quicksort (src_array, dest_remap, start, end)>
#
# The content of `src_array' is sorted using awk's rules for
# comparing values. Values with indices in range [start, end] are
# sorted.  `src_array' array is not changed.
# Instead dest_remap array is generated such that
#
#   Result:
#     src_array [dest_remap [start]] <=
#        <= src_array [dest_remap [start+1]] <=
#        <= src_array [dest_remap [start+2]] <= ... <=
#        <= src_array [dest_remap [end]]
#
# `quicksort' algorithm is used.
# Examples: see demo_quicksort and demo_quicksort2 executables
#
# =item I<quicksort_values (src_hash, dest_remap)>
#
# The same as `quicksort' described above, but hash values are sorted.
#
#   Result: 
#     src_hash [dest_remap [1]] <=
#        <= src_hash [dest_remap [2]] <=
#        <= src_hash [dest_remap [3]] <= ... <=
#        <= src_hash [dest_remap [count]]
#
# `count', a number of elements in `src_hash', is a return value.
# Examples: see demo_quicksort* executables.
#
# =item I<quicksort_indices (src_hash, dest_remap)>
#
# The same as `quicksort' described above, but hash indices are sorted.
#
#   Result:
#     dest_remap [1] <=
#        <= dest_remap [2] <=
#        <= dest_remap [3] <= ... <=
#        <= dest_remap [count]
#
# `count', a number of elements in `src_hash', is a return value.
#
# =back
#

function __quicksort (array, index_remap, start, end,
       MedIdx,Med,v,i,storeIdx)
{
	if ((end - start) <= 0)
		return

	MedIdx = int((start+end)/2)
	Med = array [index_remap [MedIdx]]

	v = index_remap [end]
	index_remap [end] = index_remap [MedIdx]
	index_remap [MedIdx] = v

	storeIdx = start
	for (i=start; i < end; ++i){
		if (array [index_remap [i]] < Med){
			v = index_remap [i]
			index_remap [i] = index_remap [storeIdx]
			index_remap [storeIdx] = v

			++storeIdx
		}
	}

	v = index_remap [storeIdx]
	index_remap [storeIdx] = index_remap [end]
	index_remap [end] = v

	__quicksort(array, index_remap, start, storeIdx-1)
	__quicksort(array, index_remap, storeIdx+1, end)
}

function quicksort (array, index_remap, start, end,             i)
{
	for (i=start; i <= end; ++i)
		index_remap [i] = i

	__quicksort(array, index_remap, start, end)
}

function quicksort_values (hash, remap_idx,
   array, remap, i, j, cnt)
{
	cnt = 0
	for (i in hash) {
		++cnt
		array [cnt] = hash [i]
		remap [cnt] = i
	}

	quicksort(array, remap_idx, 1, cnt)

	for (i=1; i <= cnt; ++i) {
		remap_idx [i] = remap [remap_idx [i]]
	}

	return cnt
}

function quicksort_indices (hash, remap_idx,
   array, i, cnt)
{
	cnt = 0
	for (i in hash) {
		++cnt
		array [cnt] = i
	}

	quicksort(array, remap_idx, 1, cnt)

	for (i=1; i <= cnt; ++i) {
		remap_idx [i] = array [remap_idx [i]]
	}

	return cnt
}
#use "quicksort.awk"
############# and below is how to use the sort functions
# This demo sorts the input lines as strings and outputs them to stdout
#
# Input files for this demo: examples/demo_quicksort.in
#
#{
#	array [++count] = $0
#}
#
#END {
#	quicksort(array, remap, 1, count)
#
#	for (i=1; i <= count; ++i){
#		print array [remap [i]]
#	}
#}

#################################################################################################################
# Start of ByteFreq Data Profiler Code
################################################################################################################
# inititalize code

BEGIN {

#### set a random seed to randomise our example strings in our reports

# user defined seed? else set one
if (seed > 0) {
     seed = seed
     # now set the seed
     srand(seed)     
   }
else {
#  uncomment below to have a fixed seed if no option set.
#     seed = 12345
#     # now set the seed
#      srand(seed)
} 


# set an index value to support bespoke sort functions
current_index = 1


# this section processes the command line options for headers, and output style
if ( header == 1 ){
	   header=1
        }
        else { 
           header=0
        } # end of the else and if
if ( report == 0 ){
           report=0
        }
        else if (report == 2) {
           report=2
        }
        else if (report == 3){
           report=3
	   tabsep = "\t"
	} 
        else {
	   report=1
	}# end of the else and if

if ( grain == "L" ){
           grain="L"
        }
        else {
           grain="H"
        } # end of the else and if


       # retrieve the current date and minute
       "date \"+%Y-%m-%d %H:%M:%S\" " | getline today
       # the above line sets the value of the variable today with the date as retrived from the command line program date. Works on nix.

} #end of BEGIN

########### START OF DATA PROCESSING - FIRST HANDLE HEADER #####################################################################
# calculate and count formats

NR == header {

# note here I add in the column numbers to the col names.
# that needs doing through adding it to a large number so the non-numeric sortation is correct in awk

  hout = ""


  for (field = 1; field <= NF ; field++) {

     clean_colname = $(field)

	  gsub(/ /,"",clean_colname)
	  gsub(/_/,"",clean_colname)
	  gsub(/\t/,"",clean_colname)
	  gsub(/^M/,"",clean_colname)

  	 if (field >1) {hout = hout FS}	

	 if (report == 2){
		hout = hout clean_colname FS "DQ_"clean_colname 
	 } else {

	    zcoln = "col_"(100000+field)
	    gsub(/^col_1/,"col_",zcoln)
		names[field]=zcoln"_"clean_colname	
	 }

  } # this is the end of the field loop 

  # if we are outputting raw with profiles data, print the header now while we've calc'd it

  if (report == 2){
	print hout
  } else if (report == 3) {
        print "report_date" tabsep "filename" tabsep "RunRowNum" tabsep "SourceRowNum" tabsep "colname" tabsep "grain" tabsep "profile" tabsep "rawval" 
  }


} # end header 



################## This is the start of the main block processing ###################

FNR > header { 
 # notice we only profile data in the rows AFTER the header, so this can help to skip headers on data produced in reports
 # I've changed this to do the find and replace on each field, as nulls were playing up if you didn't specify delim

  out = ""

  # before all the profiling, lets track files and their counts
 
  wc[FILENAME]++

  for (field = 1; field <= NF ; field++) {

		prof = $(field);

		# if the report is type 2, the doubled raw +format data, we need to add a delimiter here just after field 2
		if (field > 1) {out = out FS}

		if ( grain == "H" ){
			gsub(/[[:lower:]]/,"a",prof)
 			gsub(/[[:upper:]]/,"A",prof)
 			gsub(/[[:digit:]]/,"9",prof)
			gsub(/\t/,"T",prof)
                        gsub(/^M/,"",prof)
		}
		else {
			gsub(/[[:lower:]]+/,"a",prof)
			gsub(/[[:upper:]]+/,"A",prof)
			gsub(/[[:digit:]]+/,"9",prof)	
	                gsub(/\t/,"T",prof)
                        gsub(/^M/,"",prof)	
		}

 		# save the formatted string and increment the count in a big multidimensional array 
		
		# note, here we swap out null values for <null> so it works properly

		pattern=prof
		if ( pattern == "" ){
       			pattern="<<null>>"
        	} #end of if statement


		# here, if I'm counting the frequency I add the data to the arrary, or if a report=3, print it.
      		

		if (report == 2) {
	       		out = out $(field) FS pattern

  		} else if (report == 3) {
                        temp_colname = 100000 + field
			gsub(/^./, "" ,temp_colname)			
			
			if (names[field] != "" ) {
				temp_colname = names[field]
			} else {
				temp_colname = "col_"temp_colname
 			}
                        # REPORT == 3
                        # below we print out a line of data for each field in this row 
			print today tabsep FILENAME tabsep NR tabsep FNR tabsep temp_colname tabsep grain tabsep pattern tabsep $(field)

		} else {
                        # these are the main places we collect our profile statistics
                             # allcolumns tracks the count of a pattern in a field 

        		allcolumns[field, pattern]++
                        pattern_count = allcolumns[field, pattern]

                             # allpatterns tracks the last seen example of data that created a pattern
			     # NOTE: I'm trialling a process to randomise the examples
                             #       to do that: first example set to 100th or last row, then we back off on chance of example replacement
                        if (pattern_count  < 5) {
			    allpatterns[field, pattern] = $(field)
                            }
                        else if (pattern_count < 30) {
                            r = rand()
                            if (r < 0.05) {
                                allpatterns[field, pattern] = $(field)
                                }
                            }
                        else if (pattern_count < 1000) {
                            r = rand()
                            if (r < 0.005) {
                                allpatterns[field, pattern] = $(field)
                                }
                            }
                        else {
                            r = rand()
                            if (r < 0.0001) {
                                allpatterns[field, pattern] = $(field)
                                } 
                             } 
		}

  } # end of for field loop

file_rowcount++


# we are are not counting the formats, then just output the row
if (report == 2) {
   print out 
}


} # end of main loop

####################################### NOW PROCESS STATISTICS WE CAPUTURED  #########################################################
# process the counts
# Here we generate the sortable lineitems of our report, later we then sort these, and after output them 

END {

  # only do this post analysis where we count.  
  if (report < 2 ) {

	# loop through the number of field+patterns I have, inspect every value in the multidimensional arrary to copy out the field
        # values into a new array having a line item that I can sort then print out.

		j=1	
		for( string in allcolumns) {

                        # split the indexed fields of the array (field, pattern) into the components 
			split(string, separate, SUBSEP)

			# now I can collect/generate my info: field, arrayID, frm_string, count, num 
			# with my separated items, retrieve the count for this pattern	

			countval = allcolumns[separate[1], separate[2]]
			example = allpatterns[string]

                        # if the field name received is null, use the field count to generate a column number based one 
			if(names[separate[1]] == "" ) {
				printnames[separate[1]]="##column_"(100000000+separate[1])
			} else {
				printnames[separate[1]]=names[separate[1]]
			} # end of field name check

                        # next take the original value of the pattern counter and generate something we can sort DESC
                        sortablecount = "1"(1000000000 - countval)
                        

			linetext=FILENAME"|"printnames[separate[1]]"|"sortablecount"|"countval"|"separate[2]"\t\t"example
			lineitems[j]=linetext
	
		j++		
		} # end of loop through allcolumns

################# With the processed stats, sort them before printing  ###############################
# sort output
# I have implemented a sort function in this script to make it portable to many awk implementations

    for( i in lineitems) countof_reportitems++

    quicksort(lineitems, reportitems_idx, 1, countof_reportitems)
    
################# Now generate output report #########################################################

# print output

		# This section prints the output, either a report or a datafile as specified on the command line.
		#

		if ( report == 1 ){
                   print "#"
                   print "#"
                   print "#"
                   print "#"
		   print "# ----------------------------------------------------------------"	
		   print "# bytefreq: portable mask based data profiling for data quality   "
		   print "# ----------------------------------------------------------------"
		   print "#"
		   print "# Version: bytefreq_v1.05.awk"
                   print "# Project: https://github.com/6point6/bytefreq"
                   print "#"
                   print "#"
		   print("# Data Profiling Report: "today) 
                   print("#\t", "Lines","\t","Filename")
                   for (xfiles in wc) {
                       print("#\t",wc[xfiles], "\t", xfiles) 
                   }

		   # print("# Name of file: " FILENAME)
		   # print("# Examined rows: "(NR-header)-1  )

		   print("")
		}
		prev_finalcolname ="X"

		for (line = 1; line <= countof_reportitems; line++) {

			# now split the line to remove my internal sort key, that's the 100000000 - count thing.
			# below we retrive the line via the sorted re-index

			currline = lineitems[reportitems_idx[line]]
                        
			split(currline, linefield, "|")

                        # the linefield[] array holds the report data stored like this: 
                        #     FILENAME"|"printnames[separate[1]]"|"sortablecount"|"countval"|"separate[2]"\t\t"example
 
			# Don't forget to clean off the sort key from unheadered field names
                        finalcolname = linefield[2]
			
			# print off the final sorted report line items

			if ( report == 0 ){
				print(today"\t"linefield[1]"\t"finalcolname"\t"grain"\t"linefield[4]"\t"linefield[5])	 
				prev_finalcolname=finalcolname	
			}
        		else {
				# if we hit a new column, print header, and then record 
				if (prev_finalcolname != finalcolname){
					fhead=linefield[1]
					fcol=finalcolname	
					fpatt=linefield[5]
					gsub(/./," ",fhead)
					gsub(/^..../,"file",fhead)
					gsub(/./," ",fcol)
					gsub(/^....../,"column",fcol)
	
					print("\n"fhead"\t"fcol"\t\tcount\tpattern\t\texample")

					gsub(/./,"=",fhead)
				 	gsub(/./,"=",fcol)
					print(fhead"\t"fcol"\t\t=====\t=======\t\t=======")
                                        #print(linefield[1]"\t"finalcolname"\t"grain"\t"linefield[3]"\t"linefield[4]"\t"linefield[5])
					print(linefield[1]"\t"finalcolname"\t\t"linefield[4]"\t"linefield[5])		
					prev_finalcolname=finalcolname	
				} 
				else {
                                        #print(linefield[1]"\t"finalcolname"\t"grain"\t"linefield[3]"\t"linefield[4]"\t"linefield[5])
					print(linefield[1]"\t"finalcolname"\t\t"linefield[4]"\t"linefield[5])
					prev_finalcolname=finalcolname	
				}

		        } # end of the main reporting  if/else


		} # end of loop through sorted report lines 

  } # end of the report !=2 to check we print analysis  
} # end of END
